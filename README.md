# delivery
Goods delivery microservice

## Measurement units in DB

- weight - **g**
- dimensions - **cm**
- volume aka "size" - **cm<sup>3</sup>**
- dimensional factor - **cm<sup>3</sup>/g**

### `packages`

- `min_size` - cm<sup>3</sup>
- `max_size` - cm<sup>3</sup>
- `min_weight` - g
- `max_weight` - g

### `companies_packages`

- `dimensional_factor` - cm<sup>3</sup>/g
- `rates -> weight` - g
