pub mod routes;

use std::str::FromStr;
use std::sync::Arc;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::future;
use futures::prelude::*;
use futures_cpupool::CpuPool;
use hyper::header::Authorization;
use hyper::server::Request;
use hyper::{Delete, Get, Post};
use r2d2::{ManageConnection, Pool};

use stq_http::client::ClientHandle;
use stq_http::controller::Controller;
use stq_http::controller::ControllerFuture;
use stq_http::request_util::parse_body;
use stq_http::request_util::serialize_future;
use stq_router::RouteParser;
use stq_types::*;

use self::routes::Route;
use config;
use errors::Error;
use models::*;
use repos::repo_factory::*;
use services::user_roles::{UserRolesService, UserRolesServiceImpl};

/// Controller handles route parsing and calling `Service` layer
#[derive(Clone)]
pub struct ControllerImpl<T, M, F>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
{
    pub db_pool: Pool<M>,
    pub config: config::Config,
    pub cpu_pool: CpuPool,
    pub route_parser: Arc<RouteParser<Route>>,
    pub repo_factory: F,
    pub http_client: ClientHandle,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > ControllerImpl<T, M, F>
{
    /// Create a new controller based on services
    pub fn new(db_pool: Pool<M>, config: config::Config, cpu_pool: CpuPool, http_client: ClientHandle, repo_factory: F) -> Self {
        let route_parser = Arc::new(routes::create_route_parser());
        Self {
            db_pool,
            config,
            cpu_pool,
            route_parser,
            repo_factory,
            http_client,
        }
    }
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > Controller for ControllerImpl<T, M, F>
{
    /// Handle a request and get future response
    fn call(&self, req: Request) -> ControllerFuture {
        let headers = req.headers().clone();
        let auth_header = headers.get::<Authorization<String>>();
        let user_id = auth_header
            .map(|auth| auth.0.clone())
            .and_then(|id| i32::from_str(&id).ok())
            .map(UserId);

        debug!("User with id = '{:?}' is requesting {}", user_id, req.path());

        let user_roles_service = UserRolesServiceImpl::new(self.db_pool.clone(), self.cpu_pool.clone(), self.repo_factory.clone());

        let path = req.path().to_string();

        match (&req.method().clone(), self.route_parser.test(req.path())) {
            // GET /user_role/<user_id>
            (&Get, Some(Route::UserRole(user_id_arg))) => {
                debug!("User with id = '{:?}' is requesting  // GET /user_role/{}", user_id, user_id_arg);
                serialize_future(user_roles_service.get_roles(user_id_arg))
            }
            // POST /user_roles
            (&Post, Some(Route::UserRoles)) => {
                debug!("User with id = '{:?}' is requesting  // POST /user_roles", user_id);
                serialize_future(
                    parse_body::<NewUserRole>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /user_roles in NewUserRole failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |new_role| user_roles_service.create(new_role)),
                )
            }
            // DELETE /user_roles
            (&Delete, Some(Route::UserRoles)) => {
                debug!("User with id = '{:?}' is requesting  // DELETE /user_roles", user_id);
                serialize_future(
                    parse_body::<OldUserRole>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // DELETE /user_roles/<user_role_id> in OldUserRole failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |old_role| user_roles_service.delete(old_role)),
                )
            }
            // POST /roles/default/<user_id>
            (&Post, Some(Route::DefaultRole(user_id_arg))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /roles/default/{}",
                    user_id, user_id_arg
                );
                serialize_future(user_roles_service.create_default(user_id_arg))
            }
            // DELETE /roles/default/<user_id>
            (&Delete, Some(Route::DefaultRole(user_id_arg))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // DELETE /roles/default/{}",
                    user_id, user_id_arg
                );
                serialize_future(user_roles_service.delete_default(user_id_arg))
            }

            // Fallback
            (m, _) => Box::new(future::err(
                format_err!("Request to non existing endpoint in delivery microservice! {:?} {:?}", m, path)
                    .context(Error::NotFound)
                    .into(),
            )),
        }
    }
}
