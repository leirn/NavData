# NAVAID and  AIRPORTS REST API

This project offers a REST API to get information on:

- Airports
- Navaid

It currently relies on <https://www.github.com/davidmegginson/ourairports-data> .
Data is refreshed every 24h.

## WARNING : NO AIRAC COMPLIANCY

There is no garantee that the provided data is in any way conform to the latest AIRAC cycle.

This data MUST NOT be used to plan real life flights.

## Usage

### Environment parameters

- HOST : host the http server is listening to. Default is 127.0.0.1
- PORT : port the http server is listening to. Default is 8080
- TOKEN_LIST : a comma separated list of accepted connexion tokens for security purpose. Token muse be provided as ```navaid_auth_token```. If not set, token verification is bypassed
- DATABASE_PATH : the path to SQLite database. Defaut is ```:memory```, which means not persistent

### API

- ```GET /airport?search={query}``` : look for an airport based on ```query``` string. Answer first 100 results
- ```GET /airport/{icao}``` : look for an airport based on its ICAO code
- ```GET /navaid?search={query}``` : look for a navaid (VOR, DME, ADF...) based on ```query``` string. Answer first 100 results
- ```GET /navaid/{icao}``` : look for an navaid based on its ICAO code