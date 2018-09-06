DELETE FROM countries ;

ALTER TABLE countries ADD COLUMN parent_label VARCHAR;
ALTER TABLE countries DROP COLUMN IF EXISTS parent;

INSERT INTO countries (label, level, alpha3) VALUES ('All', 0, 'XAL');
INSERT INTO countries (parent_label, label, level, alpha3) VALUES ('All', 'Africa', 1, 'XAF');
INSERT INTO countries (parent_label, label, level, alpha3) VALUES ('All', 'Asia', 1, 'XAS');
INSERT INTO countries (parent_label, label, level, alpha3) VALUES ('All', 'Australia', 1, 'XOC');
INSERT INTO countries (parent_label, label, level, alpha3) VALUES ('All', 'Europe', 1, 'XEU');
INSERT INTO countries (parent_label, label, level, alpha3) VALUES ('All', 'North America', 1, 'XNA');
INSERT INTO countries (parent_label, label, level, alpha3) VALUES ('All', 'South America', 1, 'XSA');

INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'AF', 	'AFG', 	004, 	'Afghanistan, Islamic Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'AL', 	'ALB', 	008, 	'Albania, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Antarctica', 	'AQ', 	'ATA', 	010, 	'Antarctica (the territory South of 60 deg S)', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'DZ', 	'DZA', 	012, 	'Algeria, People`s Democratic Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'AS', 	'ASM', 	016, 	'American Samoa', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'AD', 	'AND', 	020, 	'Andorra, Principality of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'AO', 	'AGO', 	024, 	'Angola, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'AG', 	'ATG', 	028, 	'Antigua and Barbuda', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'AZ', 	'AZE', 	031, 	'Azerbaijan, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('South America', 	'AR', 	'ARG', 	032, 	'Argentina, Argentine Republic', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'AU', 	'AUS', 	036, 	'Australia, Commonwealth of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'AT', 	'AUT', 	040, 	'Austria, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'BS', 	'BHS', 	044, 	'Bahamas, Commonwealth of the', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'BH', 	'BHR', 	048, 	'Bahrain, Kingdom of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'BD', 	'BGD', 	050, 	'Bangladesh, People`s Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'AM', 	'ARM', 	051, 	'Armenia, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'BB', 	'BRB', 	052, 	'Barbados', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'BE', 	'BEL', 	056, 	'Belgium, Kingdom of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'BM', 	'BMU', 	060, 	'Bermuda', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'BT', 	'BTN', 	064, 	'Bhutan, Kingdom of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('South America', 	'BO', 	'BOL', 	068, 	'Bolivia, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'BA', 	'BIH', 	070, 	'Bosnia and Herzegovina', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'BW', 	'BWA', 	072, 	'Botswana, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Antarctica', 	'BV', 	'BVT', 	074, 	'Bouvet Island (Bouvetoya)', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('South America', 	'BR', 	'BRA', 	076, 	'Brazil, Federative Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'BZ', 	'BLZ', 	084, 	'Belize', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'IO', 	'IOT', 	086, 	'British Indian Ocean Territory (Chagos Archipelago)', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'SB', 	'SLB', 	090, 	'Solomon Islands', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'VG', 	'VGB', 	092, 	'British Virgin Islands', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'BN', 	'BRN', 	096, 	'Brunei Darussalam', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'BG', 	'BGR', 	100, 	'Bulgaria, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'MM', 	'MMR', 	104, 	'Myanmar, Union of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'BI', 	'BDI', 	108, 	'Burundi, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'BY', 	'BLR', 	112, 	'Belarus, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'KH', 	'KHM', 	116, 	'Cambodia, Kingdom of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'CM', 	'CMR', 	120, 	'Cameroon, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'CA', 	'CAN', 	124, 	'Canada', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'CV', 	'CPV', 	132, 	'Cape Verde, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'KY', 	'CYM', 	136, 	'Cayman Islands', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'CF', 	'CAF', 	140, 	'Central African Republic', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'LK', 	'LKA', 	144, 	'Sri Lanka, Democratic Socialist Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'TD', 	'TCD', 	148, 	'Chad, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('South America', 	'CL', 	'CHL', 	152, 	'Chile, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'CN', 	'CHN', 	156, 	'China, People`s Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'TW', 	'TWN', 	158, 	'Taiwan', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'CX', 	'CXR', 	162, 	'Christmas Island', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'CC', 	'CCK', 	166, 	'Cocos (Keeling) Islands', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('South America', 	'CO', 	'COL', 	170, 	'Colombia, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'KM', 	'COM', 	174, 	'Comoros, Union of the', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'YT', 	'MYT', 	175, 	'Mayotte', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'CG', 	'COG', 	178, 	'Congo, Republic of the', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'CD', 	'COD', 	180, 	'Congo, Democratic Republic of the', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'CK', 	'COK', 	184, 	'Cook Islands', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'CR', 	'CRI', 	188, 	'Costa Rica, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'HR', 	'HRV', 	191, 	'Croatia, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'CU', 	'CUB', 	192, 	'Cuba, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'CY', 	'CYP', 	196, 	'Cyprus, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'CZ', 	'CZE', 	203, 	'Czech Republic', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'BJ', 	'BEN', 	204, 	'Benin, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'DK', 	'DNK', 	208, 	'Denmark, Kingdom of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'DM', 	'DMA', 	212, 	'Dominica, Commonwealth of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'DO', 	'DOM', 	214, 	'Dominican Republic', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('South America', 	'EC', 	'ECU', 	218, 	'Ecuador, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'SV', 	'SLV', 	222, 	'El Salvador, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'GQ', 	'GNQ', 	226, 	'Equatorial Guinea, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'ET', 	'ETH', 	231, 	'Ethiopia, Federal Democratic Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'ER', 	'ERI', 	232, 	'Eritrea, State of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'EE', 	'EST', 	233, 	'Estonia, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'FO', 	'FRO', 	234, 	'Faroe Islands', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('South America', 	'FK', 	'FLK', 	238, 	'Falkland Islands (Malvinas)', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Antarctica', 	'GS', 	'SGS', 	239, 	'South Georgia and the South Sandwich Islands', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'FJ', 	'FJI', 	242, 	'Fiji, Republic of the Fiji Islands', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'FI', 	'FIN', 	246, 	'Finland, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'AX', 	'ALA', 	248, 	'Åland Islands', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'FR', 	'FRA', 	250, 	'France, French Republic', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('South America', 	'GF', 	'GUF', 	254, 	'French Guiana', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'PF', 	'PYF', 	258, 	'French Polynesia', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Antarctica', 	'TF', 	'ATF', 	260, 	'French Southern Territories', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'DJ', 	'DJI', 	262, 	'Djibouti, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'GA', 	'GAB', 	266, 	'Gabon, Gabonese Republic', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'GE', 	'GEO', 	268, 	'Georgia', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'GM', 	'GMB', 	270, 	'Gambia, Republic of the', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'PS', 	'PSE', 	275, 	'Palestinian Territory, Occupied', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'DE', 	'DEU', 	276, 	'Germany, Federal Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'GH', 	'GHA', 	288, 	'Ghana, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'GI', 	'GIB', 	292, 	'Gibraltar', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'KI', 	'KIR', 	296, 	'Kiribati, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'GR', 	'GRC', 	300, 	'Greece, Hellenic Republic', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'GL', 	'GRL', 	304, 	'Greenland', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'GD', 	'GRD', 	308, 	'Grenada', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'GP', 	'GLP', 	312, 	'Guadeloupe', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'GU', 	'GUM', 	316, 	'Guam', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'GT', 	'GTM', 	320, 	'Guatemala, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'GN', 	'GIN', 	324, 	'Guinea, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('South America', 	'GY', 	'GUY', 	328, 	'Guyana, Co-operative Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'HT', 	'HTI', 	332, 	'Haiti, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Antarctica', 	'HM', 	'HMD', 	334, 	'Heard Island and McDonald Islands', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'VA', 	'VAT', 	336, 	'Holy See (Vatican City State)', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'HN', 	'HND', 	340, 	'Honduras, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'HK', 	'HKG', 	344, 	'Hong Kong, Special Administrative Region of China', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'HU', 	'HUN', 	348, 	'Hungary, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'IS', 	'ISL', 	352, 	'Iceland, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'IN', 	'IND', 	356, 	'India, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'ID', 	'IDN', 	360, 	'Indonesia, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'IR', 	'IRN', 	364, 	'Iran, Islamic Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'IQ', 	'IRQ', 	368, 	'Iraq, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'IE', 	'IRL', 	372, 	'Ireland', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'IL', 	'ISR', 	376, 	'Israel, State of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'IT', 	'ITA', 	380, 	'Italy, Italian Republic', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'CI', 	'CIV', 	384, 	'Côte d`Ivoire, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'JM', 	'JAM', 	388, 	'Jamaica', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'JP', 	'JPN', 	392, 	'Japan', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'KZ', 	'KAZ', 	398, 	'Kazakhstan, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'JO', 	'JOR', 	400, 	'Jordan, Hashemite Kingdom of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'KE', 	'KEN', 	404, 	'Kenya, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'KP', 	'PRK', 	408, 	'Korea, Democratic People`s Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'KR', 	'KOR', 	410, 	'Korea, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'KW', 	'KWT', 	414, 	'Kuwait, State of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'KG', 	'KGZ', 	417, 	'Kyrgyz Republic', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'LA', 	'LAO', 	418, 	'Lao People`s Democratic Republic', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'LB', 	'LBN', 	422, 	'Lebanon, Lebanese Republic', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'LS', 	'LSO', 	426, 	'Lesotho, Kingdom of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'LV', 	'LVA', 	428, 	'Latvia, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'LR', 	'LBR', 	430, 	'Liberia, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'LY', 	'LBY', 	434, 	'Libyan Arab Jamahiriya', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'LI', 	'LIE', 	438, 	'Liechtenstein, Principality of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'LT', 	'LTU', 	440, 	'Lithuania, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'LU', 	'LUX', 	442, 	'Luxembourg, Grand Duchy of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'MO', 	'MAC', 	446, 	'Macao, Special Administrative Region of China', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'MG', 	'MDG', 	450, 	'Madagascar, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'MW', 	'MWI', 	454, 	'Malawi, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'MY', 	'MYS', 	458, 	'Malaysia', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'MV', 	'MDV', 	462, 	'Maldives, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'ML', 	'MLI', 	466, 	'Mali, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'MT', 	'MLT', 	470, 	'Malta, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'MQ', 	'MTQ', 	474, 	'Martinique', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'MR', 	'MRT', 	478, 	'Mauritania, Islamic Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'MU', 	'MUS', 	480, 	'Mauritius, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'MX', 	'MEX', 	484, 	'Mexico, United Mexican States', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'MC', 	'MCO', 	492, 	'Monaco, Principality of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'MN', 	'MNG', 	496, 	'Mongolia', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'MD', 	'MDA', 	498, 	'Moldova, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'ME', 	'MNE', 	499, 	'Montenegro, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'MS', 	'MSR', 	500, 	'Montserrat', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'MA', 	'MAR', 	504, 	'Morocco, Kingdom of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'MZ', 	'MOZ', 	508, 	'Mozambique, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'OM', 	'OMN', 	512, 	'Oman, Sultanate of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'NA', 	'NAM', 	516, 	'Namibia, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'NR', 	'NRU', 	520, 	'Nauru, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'NP', 	'NPL', 	524, 	'Nepal, State of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'NL', 	'NLD', 	528, 	'Netherlands, Kingdom of the', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'AN', 	'ANT', 	530, 	'Netherlands Antilles', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'CW', 	'CUW', 	531, 	'Curaçao', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'AW', 	'ABW', 	533, 	'Aruba', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'SX', 	'SXM', 	534, 	'Sint Maarten (Netherlands)', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'BQ', 	'BES', 	535, 	'Bonaire, Sint Eustatius and Saba', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'NC', 	'NCL', 	540, 	'New Caledonia', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'VU', 	'VUT', 	548, 	'Vanuatu, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'NZ', 	'NZL', 	554, 	'New Zealand', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'NI', 	'NIC', 	558, 	'Nicaragua, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'NE', 	'NER', 	562, 	'Niger, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'NG', 	'NGA', 	566, 	'Nigeria, Federal Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'NU', 	'NIU', 	570, 	'Niue', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'NF', 	'NFK', 	574, 	'Norfolk Island', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'NO', 	'NOR', 	578, 	'Norway, Kingdom of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'MP', 	'MNP', 	580, 	'Northern Mariana Islands, Commonwealth of the', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'UM', 	'UMI', 	581, 	'United States Minor Outlying Islands', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'FM', 	'FSM', 	583, 	'Micronesia, Federated States of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'MH', 	'MHL', 	584, 	'Marshall Islands, Republic of the', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'PW', 	'PLW', 	585, 	'Palau, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'PK', 	'PAK', 	586, 	'Pakistan, Islamic Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'PA', 	'PAN', 	591, 	'Panama, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'PG', 	'PNG', 	598, 	'Papua New Guinea, Independent State of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('South America', 	'PY', 	'PRY', 	600, 	'Paraguay, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('South America', 	'PE', 	'PER', 	604, 	'Peru, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'PH', 	'PHL', 	608, 	'Philippines, Republic of the', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'PN', 	'PCN', 	612, 	'Pitcairn Islands', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'PL', 	'POL', 	616, 	'Poland, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'PT', 	'PRT', 	620, 	'Portugal, Portuguese Republic', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'GW', 	'GNB', 	624, 	'Guinea-Bissau, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'TL', 	'TLS', 	626, 	'Timor-Leste, Democratic Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'PR', 	'PRI', 	630, 	'Puerto Rico, Commonwealth of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'QA', 	'QAT', 	634, 	'Qatar, State of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'RE', 	'REU', 	638, 	'Reunion', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'RO', 	'ROU', 	642, 	'Romania', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'RU', 	'RUS', 	643, 	'Russian Federation', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'RW', 	'RWA', 	646, 	'Rwanda, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'BL', 	'BLM', 	652, 	'Saint Barthelemy', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'SH', 	'SHN', 	654, 	'Saint Helena', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'KN', 	'KNA', 	659, 	'Saint Kitts and Nevis, Federation of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'AI', 	'AIA', 	660, 	'Anguilla', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'LC', 	'LCA', 	662, 	'Saint Lucia', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'MF', 	'MAF', 	663, 	'Saint Martin', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'PM', 	'SPM', 	666, 	'Saint Pierre and Miquelon', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'VC', 	'VCT', 	670, 	'Saint Vincent and the Grenadines', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'SM', 	'SMR', 	674, 	'San Marino, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'ST', 	'STP', 	678, 	'São Tomé and Príncipe, Democratic Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'SA', 	'SAU', 	682, 	'Saudi Arabia, Kingdom of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'SN', 	'SEN', 	686, 	'Senegal, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'RS', 	'SRB', 	688, 	'Serbia, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'SC', 	'SYC', 	690, 	'Seychelles, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'SL', 	'SLE', 	694, 	'Sierra Leone, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'SG', 	'SGP', 	702, 	'Singapore, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'SK', 	'SVK', 	703, 	'Slovakia (Slovak Republic)', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'VN', 	'VNM', 	704, 	'Vietnam, Socialist Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'SI', 	'SVN', 	705, 	'Slovenia, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'SO', 	'SOM', 	706, 	'Somalia, Somali Republic', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'ZA', 	'ZAF', 	710, 	'South Africa, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'ZW', 	'ZWE', 	716, 	'Zimbabwe, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'ES', 	'ESP', 	724, 	'Spain, Kingdom of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'SS', 	'SSD', 	728, 	'South Sudan', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'SD', 	'SDN', 	729, 	'Sudan, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'EH', 	'ESH', 	732, 	'Western Sahara', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('South America', 	'SR', 	'SUR', 	740, 	'Suriname, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'SJ', 	'SJM', 	744, 	'Svalbard & Jan Mayen Islands', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'SZ', 	'SWZ', 	748, 	'Swaziland, Kingdom of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'SE', 	'SWE', 	752, 	'Sweden, Kingdom of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'CH', 	'CHE', 	756, 	'Switzerland, Swiss Confederation', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'SY', 	'SYR', 	760, 	'Syrian Arab Republic', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'TJ', 	'TJK', 	762, 	'Tajikistan, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'TH', 	'THA', 	764, 	'Thailand, Kingdom of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'TG', 	'TGO', 	768, 	'Togo, Togolese Republic', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'TK', 	'TKL', 	772, 	'Tokelau', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'TO', 	'TON', 	776, 	'Tonga, Kingdom of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'TT', 	'TTO', 	780, 	'Trinidad and Tobago, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'AE', 	'ARE', 	784, 	'United Arab Emirates', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'TN', 	'TUN', 	788, 	'Tunisia, Tunisian Republic', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'TR', 	'TUR', 	792, 	'Turkey, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'TM', 	'TKM', 	795, 	'Turkmenistan', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'TC', 	'TCA', 	796, 	'Turks and Caicos Islands', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'TV', 	'TUV', 	798, 	'Tuvalu', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'UG', 	'UGA', 	800, 	'Uganda, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'UA', 	'UKR', 	804, 	'Ukraine', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'MK', 	'MKD', 	807, 	'Macedonia, The Former Yugoslav Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'EG', 	'EGY', 	818, 	'Egypt, Arab Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'GB', 	'GBR', 	826, 	'United Kingdom of Great Britain & Northern Ireland', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'GG', 	'GGY', 	831, 	'Guernsey, Bailiwick of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'JE', 	'JEY', 	832, 	'Jersey, Bailiwick of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Europe', 	'IM', 	'IMN', 	833, 	'Isle of Man', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'TZ', 	'TZA', 	834, 	'Tanzania, United Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'US', 	'USA', 	840, 	'United States of America', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('North America', 	'VI', 	'VIR', 	850, 	'United States Virgin Islands', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'BF', 	'BFA', 	854, 	'Burkina Faso', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('South America', 	'UY', 	'URY', 	858, 	'Uruguay, Eastern Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'UZ', 	'UZB', 	860, 	'Uzbekistan, Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('South America', 	'VE', 	'VEN', 	862, 	'Venezuela, Bolivarian Republic of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'WF', 	'WLF', 	876, 	'Wallis and Futuna', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Oceania', 	'WS', 	'WSM', 	882, 	'Samoa, Independent State of', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Asia', 	'YE', 	'YEM', 	887, 	'Yemen', 2);
INSERT INTO countries (parent_label, alpha2, alpha3, numeric, label, level) VALUES ('Africa', 	'ZM', 	'ZMB', 	894, 	'Zambia, Republic of', 2);