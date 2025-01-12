<h1 align='center'>Hecate</h1>

<p align=center>OpenStreetMap Inspired Data Storage Backend Focused on Performance and GeoJSON Interchange</p>

<p align="center">
  <a href="https://circleci.com/gh/ingalls/Hecate/tree/master"><img src="https://circleci.com/gh/ingalls/Hecate/tree/master.svg?style=shield"/></a>
  <a href="https://crates.io/crates/hecate"><img src="https://img.shields.io/crates/v/hecate.svg"/></a>
</p>

<img src="https://i.imgur.com/xLjiTqL.jpg"/>

## Hecate Feature Comparison

| Feature               | Hecate             | ESRI MapServer     | OSM Backend        |
| --------------------- | ------------------ | ------------------ | ------------------ |
| Vector Tile Creation  | :heavy_check_mark: | :heavy_check_mark: | :x:                |
| Streaming Query API   | :heavy_check_mark: | :x:                | :x:                |
| Multi User Support    | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| Feature History       | :heavy_check_mark: | :x:                | :heavy_check_mark: |
| Atomic API Operations | :heavy_check_mark: | :heavy_check_mark: | :x:                |
| GeoJSON-LD Based API  | :heavy_check_mark: | :x:                | :heavy_check_mark: |
| Mapbox GL JS Styling  | :heavy_check_mark: | :heavy_check_mark: | :x:                |
| Integrated Data Stats | :heavy_check_mark: | :heavy_check_mark: | :x:                |

## Table Of Contents

<details>

1. [Brief](#brief)
1. [Why Use Hecate](#why-use-hecate)
2. [Table of Contents](#table-of-contents)
3. [Related Libraries](#related-libraries)
4. [Build Environment](#build-environment)
5. [Docker File](#docker-file-coverage-tests)
6. [Feature Format](#feature-format)
7. [Server](#server)
    - [Database Connection](#database)
    - [JSON Validation](#json-validation)
    - [Custom Authentication](#custom-authentication)
8. [API](#api)
    - [User Options](#user-options)
    - [Meta](#meta)
    - [Data Stats](#data-stats)
    - [Admin Interface](#admin-interface)
    - [Schema](#schema)
    - [Webhooks](#webhooks)
    - [Authentication](#authentication)
    - [Styles](#styles)
    - [Vector Tiles](#vector-tiles)
    - [Downloading Via Clone](#downloading-via-clone)
    - [Downloading Via Query](#downloading-via-query)
    - [Boundaries](#boundaries)
    - [Downloading Individual Features](#downloading-individual-features)
    - [Downloading Multiple Features](#downloading-multiple-features)
    - [Feature Creation](#feature-creation)
    - [Deltas](#deltas)
    - [OpenStreetMap API](#openstreetmap-api)

</details>

## Related Libraries

<details>

- [HecateJS](https://github.com/mapbox/HecateJS) Javascript Library & CLI Tool for interacting with the Hecate API
- [Hecate-Example](https://github.com/ingalls/hecate-example) Script for importing some fake data for testing

Built something cool that uses the Hecate API? Let us know!

</details>

## Build Environment

<details>

- Start by installing Rust from [rust-lang.org](https://www.rust-lang.org/en-US/), this will install the current stable version

```bash
curl https://sh.rustup.rs -sSf | sh
```

- Source your `bashrc/bash_profile` to update your `PATH` variable

```bash
source ~/.bashrc        # Most Linux Distros, some OSX
source ~/.bash_profile  # Most OSX, some Linux Distros
```

- Install the `nightly-2019-06-01` build of rust, `Rocket`, the web-framework relies on some advanced compiler options not yet included in the default build.

```bash
rustup install nightly-2019-06-01
rustup default nightly-2019-06-01
```

- Download and compile the project and all of it's libraries

```bash
cargo build
```

- Ensure you have database dependencies `postgres` and `postgis` installed.

- Create the `hecate` database using the provided schema file. These instructions assume you have set up a role `postgres` with sufficient privileges.

```bash
echo "CREATE DATABASE hecate;" | psql -U postgres

psql -U postgres -f src/schema.sql hecate
```

- This step will also create a database role called `hecate` and `hecate_read`. If
the connection fails due to authentication, your pg_hba file may not be set up
to trust local connections.

Your pb_hba file location can be found using `echo "show hba_file;" | psql -U postgres`

Replace the file with the following:

```
local all postgres trust
local all all trust
host all all 127.0.0.1/32 trust
host all all ::1/128 trust
host replication postgres samenet trust
```

- Start the server

```bash
cargo run
```

- Test it is working - should respond with `HTTP200`

```bash
curl localhost:8000
```

You will now have an empty database which can be populated with your own data/user accounts.

If you want to populate the database with sample data for testing, [ingalls/hecate-example](https://github.com/ingalls/hecate-example)
has a selection of scripts to populate the database with test data.

</details>

## Docker File (Coverage Tests)

<details>

The Docker file is designed to give the user a testing environment to easily run rust tests.

Install docker and then run

```
docker build .

docker run {{HASH FROM ABOVE}}
```

</details>

## Feature Format

<details>

Hecate is designed as a GeoJSON first interchange and uses [standard GeoJSON](http://geojson.org/) with a couple additions
and exceptions as outlined below.

*Supported Geometry Types*
- `Point`
- `MultiPoint`
- `LineString`
- `MultiLineString`
- `Polygon`
- `MultiPolygon`

*Unsupported Geometry Types*
- `GeometryCollection`

### Additional Members

The following table outlines top-level members used by hecate to handle feature creation/modification/deletion.

Key/Value pairs in the `.properties` of a given feature are _never_ directly used by the server and are simply
passed through to the storage backend. This prevents potential conflicts between user properties and required
server members.

| Member    | Notes |
| :-------: | ----- |
| `id`      | The unique integer `id` of a given feature. Note that all features get a unique id accross GeoJSON Geometry Type |
| `version` | The version of a given feature, starts at `1` for a newly created feature |
| `action`  | Only used for uploads, the desired action to be performed. One of `create`, `modify`, `delete`, or `restore` |
| `key`     | `Optional` A String containing a value that hecate will ensure remains unique across all features. Can be a natural id (wikidata id, PID, etc), computed property hash, geometry hash etc. The specifics are left up to the client. Should an attempt at importing a Feature with a differing `id` but identical `key` be made, the feature with will be rejected, ensuring the uniqueness of the `key` values. By default this value will be `NULL`. Duplicate `NULL` values are allowed.
| `force`   | `Optional` Boolean allowing a user to override version locking and force UPSERT a feature. Disabled by default |

### Examples

#### Downloaded Features

```JSON
{
    "id": 123,
    "key": "Q1234",
    "version": 2,
    "type": "Feature",
    "properties": {
        "shop": true,
        "name": "If Pigs Could Fly"
    },
    "geometry": {
        "type": "Point",
        "coordinates": [0,0]
    }
}
```

Downloaded Features will return the integer `id` of the feature, the current `version` and the user supplied `properties` and `geojson`.
`action` is not applicable for downloaded features, it is only used on upload.

#### Create Features

```JSON
{
    "action": "create",
    "key": "11-22-33-44-1234",
    "type": "Feature",
    "properties": {
        "shop": true,
        "name": "If Pigs Could Fly"
    },
    "geometry": {
        "type": "Point",
        "coordinates": [0,0]
    }
}
```

A features being uploaded for creation must have the `action: create` property. Since an `id` and `version` have not yet been
assigned they must be omitted. Should an `id` be included it will be ignored. Adding a `version` property will throw an error.

Optionally create actions can use the `force: true` option to perform an `UPSERT` like option. In this mode the uploader must
specify the `key` value. Hecate will then `INSERT` the feature if the `key` value is new, if the `key` is already existing, the
existing feature will be overwritten with the forced feature. Note that this mode ignores version checks and is therefore unsafe.

Force Prerequisites
- Disabled by default, must be explicitly enabled via [Custom Authentication](#custom-authentication)
- Can only be performed on a feature with `action: create`
- Must specify a valid `key`

#### Modify Features

```JSON
{
    "id": 123,
    "key": "Fn4aAsJ30",
    "version": 1,
    "action": "modify",
    "type": "Feature",
    "properties": {
        "shop": true,
        "name": "If Pigs Could Fly"
    },
    "geometry": {
        "type": "Point",
        "coordinates": [0,0]
    }
}
```

A feature being uploaded for modification must have the `action: modify` as well as the `id` and `version` property. The `id` is the integer id of the feature to modify and the `version` property is the
current version of the feature as stored by the server. If the version uploaded does not match the version that the server has stored, the modify will fail. This prevents consecutive edits from conflicting.

Note that the modify operation is _not a delta operation_ and the full feature with the complete Geometry & All Properties must be included with each modify.

Also note that since the `id` pool is shared accross geometry types, an id is allowed to change it's geometry type. eg. If `id: 1` is a `Point` and then a subsequent `action: modify` with a `Polygon` geometry is performed, `id: 1` is allowed to switch to the new `Polygon` type.

#### Delete Features

```JSON
{
    "id": 123,
    "version": 1,
    "action": "delete",
    "type": "Feature",
    "properties": null,
    "geometry": null
}
```

A feature being uploaded for deletion must have the `action: delete` as well as the `id` and `version` property. See _Modify Features_ above for an explanation of those properties.

Note the `properties` and `geometry` attributes must still be included. They can be set to `null` or be their previous value. They will be ignored.

#### Restore Features

```JSON
{
    "id": 123,
    "version": 2,
    "key": "new-optional-key",
    "action": "restore",
    "type": "Feature",
    "properties": {
        "test": true,
        "random_array": [1, 2, 3]
    },
    "geometry": {
        "type": "Point",
        "coordinates": [ 12.34, 56.78 ]
    }
}
```

A feature being uploaded for restoration must have the `action: restore` as well as the `id` and `version` properties. A `restore` action is just a `modify` on a deleted feature.

Restore places the new given geometry/properties at the id specified. It does not automatically roll back the feature to it's state before deletion, if this is desired, one
must use the Feature History API to get the state before deletion and then perform the `restore` action.

Note: Restore will throw an error if an feature still exists.

</details>

## Server

This section of the guide goes over various options for launching the server

Hecate can be launched with default options with

```
cargo run
```

### Database

<details>

#### Main Connection

By default hecate will attempt to connect to `hecate@localhost:5432/hecate` for read/write
operations and simultaneously connect to `hecate_read@localhost:5432/hecate` for
sandboxed read only operations.

Note that only postgres w/ postgis enabled is supported.

This database should be created prior to launching hecate. For instructions on setting up the database
see the [Build Environment](#build-environment) section of this doc.

A custom database name, postgres user or port can be specified using the database flag.

*Example*

```bash
cargo run -- --database "<USER>:<PASSWORD>@<HOST>/<DATABASE>"

cargo run -- --database "<USER>@<HOST>/<DATABASE>"
```

#### Sandbox Connection

A second read-only account should also be created with permissions to SELECT from the
`geo` & `deltas` table. This endpoint will only be used for the `query` endpoint, which
allows arbitrary user query execution. A sample implementation can be found in the `schema.sql` document

Note: It is up to the DB Admin to ensure the permissions are limited in scope for this user. Hecate will
expose access to this user via the query endpoint.

If multiple instances of `database_sandbox` are present, hecate will load balance accross the multiple read instances.

```bash
cargo run -- --database_sandbox "<USER>:<PASSWORD>@<HOST>/<DATABASE>"

cargo run -- --database_sandbox "<USER>@<HOST>/<DATABASE>"

cargo run -- --database_sandbox "<USER>@<HOST>/<DATABASE>" --database_sandbox "<USER>@<HOST>/<DATABASE>"
```

#### Replica Connection [optional]

Finally, optionally multiple `--database_replica` conncetions can be specified which hecate
will use to load balance read traffic accross, alleviating capacity on the master db for write operations.

```bash
cargo run -- --database_replica "<USER>:<PASSWORD>@<HOST>/<DATABASE>"

cargo run -- --database_replica "<USER>@<HOST>/<DATABASE>"

cargo run -- --database_replica"<USER>@<HOST>/<DATABASE>" --database_replica "<USER>@<HOST>/<DATABASE>"
```


</details>

### JSON Validation

<details>

By default Hecate will allow any property on a given GeoJSON feature, including nestled arrays, maps, etc.

A custom property validation file can be specified using the schema flag.

*Example*

```bash
cargo run -- --schema <PATH-TO-SCHEMA>.json
```

Note hecate currently supports the JSON Schema draft-04. Once draft-06/07 support lands in
[valico](https://github.com/rustless/valico) we can support newer versions of the spec.

</details>

### Custom Authentication

<details>

By default the Hecate API is most favourable to a crowd-sourced data server. Any users
can access the data/vector tiles, users can create & manage data, and admins
can manage user accounts.

This provides a middle ground for most users but all endpoints are entirely configurable
and can run from a fully open server to fully locked down.

If the default values aren't suitable for what you intend, passing in an authentication
configuration JSON document will override the defaults.

*Example*

```
cargo run -- --auth path/to/auth.json
```

__Contents of auth.json__
```
{
    "endpoints": {
        "server": "public",
        "schema": null,
        "mvt": {
            "get": "user",
            "regen": "admin",
            "meta": null
        },
        "users": {
            "info": "admin",
            "create": "admin",
            "create_session": null
        },

        ....

    }
}
```

It is important to note that if custom authentication is used, _every_ category must be either disabled or have
an option for every sub category within it set. One cannot conditionally override only a subset of of the default options. This is for the security of private
servers, since adding a new API endpoint is a non-breaking change, the server checks that you have specified
a policy for every endpoint or are happy with just the defaults before it will start.

IE:

The below schema is invalid. Each category (schema, user, style) etc. must be specified as disabled or
have a map containing the auth for each subkey.

```
{
    "endpoint": {
        "schema": null
    }
}
```

#### Behavior Types

| Type      | Description |
| --------- | ----------- |
| `"public"`  | Allow any authenticated or unauthenticated user access |
| `"admin"`   | Allow only users with the `access: 'admin'` property on their user accounts access |
| `"user"`    | Allow any user access to the endpoint |
| `"self"`    | Only the specific user or an admin can edit their own metadata |
| `"null"`    | Disable all access to the endpoint (Must be explicitly `null` |

#### Endpoint Lookup

| Example Endpoint                      | Config Name               | Default       | Supported Behaviors       | Notes |
| ------------------------------------- | ------------------------- | :-----------: | ------------------------- | :---: |
| `GET /api`                            | `server`                  | `public`      | All                       |       |
| **Server Meta**                       | `meta`                    |               | `null`                    | 2     |
| `GET /api/meta`                       | `meta::list`              | `public`      | All                       |       |
| `GET /api/meta/<key>`                 | `meta::get`               | `public`      | All                       |       |
| `POST /api/meta/<key>`                | `meta::set`               | `admin`       | `user`, `admin`, `null`   |       |
| **JSON Schema**                       | `schema`                  |               | `null`                    | 2     |
| `GET /api/schema`                     | `schema::get`             | `public`      | All                       |       |
| **Custom Auth JSON**                  | `auth`                    |               | `null`                    | 2     |
| `GET /api/auth`                       | `auth::get`               | `public`      | All                       |       |
| **Mapbox Vector Tiles**               | `mvt`                     |               | `null`                    | 2     |
| `DELETE /api/tiles`                   | `mvt::delete`             | `admin`       | All                       |       |
| `GET /api/tiles/<z>/<x>/<y>`          | `mvt::get`                | `public`      | All                       |       |
| `GET /api/tiles/<z>/<x>/<y>/regen`    | `mvt::regen`              | `user`        | All                       |       |
| `GET /api/tiles/<z>/<x>/<y>/meta`     | `mvt::meta`               | `public`      | All                       |       |
| **Users**                             | `user`                    |               | `null`                    | 2     |
| `GET /api/users`                      | `user::list`              | `user`        | All                       |       |
| `GET /api/user/info`                  | `user::info`              | `self`        | `self`, `admin`, `null`   |       |
| `GET /api/create`                     | `user::create`            | `public`      | All                       |       |
| `GET /api/create/session`             | `user::create_session`    | `self`        | `self`, `admin`, `null`   |       |
| **Mapbox GL Styles**                  | `style`                   |               | `null`                    | 2     |
| `POST /api/style`                     | `style::create`           | `self`        | `self`, `admin`, `null`   |       |
| `PATCH /api/style`                    | `style::patch`            | `self`        | `self`, `admin`, `null`   |       |
| `POST /api/style/<id>/public`         | `style::set_public`       | `self`        | All                       |       |
| `POST /api/style/<id>/private`        | `style::set_private`      | `self`        | `self`, `admin`, `null`   |       |
| `DELETE /api/style/<id>`              | `style::delete`           | `self`        | `self`, `admin`, `null`   |       |
| `GET /api/style/<id>`                 | `style::get`              | `public`      | All                       | 1     |
| `GET /api/styles`                     | `style::list`             | `public`      | All                       | 1     |
| **Deltas**                            | `delta`                   |               | `null`                    | 2     |
| `GET /api/delta/<id>`                 | `delta::get`              | `public`      | All                       |       |
| `GET /api/deltas`                     | `delta::list`             | `public`      | All                       |       |
| **Webhooks**                          | `webhooks`                |               | `null`                    | 2     |
| `GET /api/webhooks`                   | `webhooks::list`          | `admin`       | All                       |       |
| `GET /api/webhooks/<id>`              | `webhooks::list`          | `admin`       | All                       |       |
| `GET /api/webhooks/<id>`              | `webhooks::delete`        | `admin`       | All                       |       |
| `POST /api/webhooks/<id>`             | `webhooks::update`        | `admin`       | All                       |       |
| **Data Stats**                        | `stats`                   | `public`      | All                       |       |
| `GET /api/data/stats`                 | `stats::get`              | `public`      | All                       |       |
| `GET /api/data/bounds/<id>/stats`     | `stats::bounds`           | `public`      | All                       |       |
| **Features**                          | `feature`                 |               | `null`                    | 2     |
| `POST /api/data/feature(s)`           | `feature::create`         | `user`        | `user`, `admin`, `null`   |       |
| `GET /api/data/feature/<id>`          | `feature::get`            | `public`      | All                       |       |
| `GET /api/data/feature/<id>/history`  | `feature::history`        | `public`      | All                       |       |
| `POST /api/data/feature(s) w/ `force` | `feature::force`          | `admin`       | `user`, `admin`, `null`   |       |
| **Clone**                             | `clone`                   |               | `null`                    | 2     |
| `GET /api/data/clone`                 | `clone::get`              | `user`        | All                       |       |
| `GET /api/data/query`                 | `clone::query`            | `user`        | All                       |       |
| **Bounds**                            | `bounds`                  |               | `null`                    | 2     |
| `GET /api/bounds`                     | `bounds::list`            | `public`      | All                       |       |
| `GET /api/bounds/<id>`                | `bounds::get`             | `public`      | All                       |       |
| `POST /api/bounds/<id>`               | `bounds::create`          | `admin`       | All                       |       |
| `DELETE /api/bounds/<id>`             | `bounds:delete`           | `admin`       | All                       |       |
| **OpenStreetMap Shim**                | `osm`                     |               | `null`                    | 2     |
| `GET /api/0.6/map`                    | `osm::get`                | `public`      | All                       | 3     |
| `PUT /api/0.6/changeset/<id>/upload`  | `osm::create`             | `user`        | `user`, `admin`, `null`   | 3     |

*Notes*

1. This only affectes `public` styles. The `private` attribute on a style overrides this. A `private` style can _never_ be seen publicly regardless of this setting.
2. This is a category, the only valid option is `null` this will disable access to the endpoint entirely
3. OSM software expects the authentication on these endpoints to mirror OSM. Setting these to a non-default option is supported but will likely have unpredicable
support when using OSM software. If you are running a private server you should disable OSM support entirely.

</details>

## API

<h3 align='center'>Index</h3>

#### `GET` `/`

HTTP Healthcheck URL, currently returns `Hello World!`

*Example*

```bash
curl -X GET 'http://localhost:8000/'
```

---

<h3 align='center'>Admin Interface</h3>

View the Admin Interface in your browser by pointing to `127.0.0.1:8000/admin/index.html`

---

<h3 align='center'>Meta</h3>

<details>

#### `GET` `/api`

Return a JSON object containing metadata about the server

*Example*

```bash
curl -X GET 'http://localhost:8000/api'
```

</details>

---

<h3 align='center'>Data Stats</h3>

<details>

Note: Analyze stats depend on the database having `ANALYZE` run.
For performance reasons these stats are calculated from ANALYZEd stats
where possible to ensure speedy results. For more up to date stats,
ensure your database is running `ANALYZE` more often. This can be done
manually in the database or by using the `/api/data/stats/regen` API.

#### `GET` `/api/data/stats`

Return a JSON object containing statistics and metadata about the
geometries stored in the server

*Example*

```bash
curl -X GET 'http://localhost:8000/api/data/stats'
```

---

#### `GET` `/api/data/stats/regen`

Perform an `ANAYLZE` call on the `geo` table to update
the global stats.

*Example*

```bash
curl -X GET 'http://localhost:8000/api/data/stats/regen'
```

</details>

---

<h3 align='center'>Styles</h3>

<details>

#### `GET` `/api/styles`

Return an array containing a reference to every public style

*Example*

```bash
curl -X GET 'http://localhost:8000/api/styles'
```

---

#### `GET` `/api/styles/<user id>`

Return an array containing styles owned by a particular user.

By default any request will only return the public styles for a given user.

If an authenticated user requests their own styles, it will return their public and private styles.

*Options*

| Option | Notes |
| :----: | ----- |
| `<user id>` | `REQUIRED` Numeric ID of the user to get styles from |

*Example*

Return only public styles of user 1

```bash
curl -X GET 'http://localhost:8000/api/styles/1'
```

User requesting their own styles will get public & private styles

```bash
curl -X GET \
    -u 'username:password' \
    'http://localhost:8000/api/styles/1'
```

---

#### `POST` `/api/style`

Create a new private style attached to the authenticated user

*Example*

```bash
curl \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"name": "Name of this particular style", "style": "Mapbox Style Object Here"}' \
    -u 'username:password' \
    'http://localhost:8000/api/style'
```

---

#### `DELETE` `/api/style/<id>`

Delete a particular style by id. Users must be authorized and
can only delete styles created by them.

*Options*

| Option | Notes |
| :----: | ----- |
| `<id>` | `REQUIRED` Numeric ID of a given style to delete |

*Example*

```bash
curl -X DELETE 'http://localhost:8000/api/style/1'
```

---

#### `GET` `/api/style/<id>`

Get a particular style by id, public styles can be requested unauthenticated,
private styles can only be obtained by the corresponding user making the request.

*Options*

| Option | Notes |
| :----: | ----- |
| `<id>` | `REQUIRED` Numeric ID of a given style to download |

*Example*

```bash
curl -X GET 'http://localhost:8000/api/style/1'
```

---

#### `PATCH` `/api/style/<id>`

Update a style - auth required - users can only update their own styles

*Options*

| Option | Notes |
| :----: | ----- |
| `<id>` | `REQUIRED` Numeric ID of a given style to download |

*Example*

```bash
curl \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"name": "New Name", "style": "New Mapbox Style Object Here"}' \
    -u 'username:password' \
    'http://localhost:8000/api/style/1'
```

---

#### `POST` `/api/style/<id>/private`

Update a public style and mark it as private.

Note: Once a style is public other users may have cloned it. This will not
affect cloned styles that were made when it was public.

*Options*

| Option | Notes |
| :----: | ----- |
| `<id>` | `REQUIRED` Numeric ID of a given style to download |

*Example*

```bash
curl -X POST \
    -u 'username:password' \
    'http://localhost:8000/api/style/1/private'
```

---

#### `POST` `/api/style/<id>/public`

Update a style to make it public.

It will then appear to all users in the global styles list
and other users will be able to download, clone, and use it

*Options*

| Option | Notes |
| :----: | ----- |
| `<id>` | `REQUIRED` Numeric ID of a given style to download |

*Example*

```bash
curl -X POST \
    -u 'username:password' \
    'http://localhost:8000/api/style/1/public'
```

</details>

---

<h3 align='center'>Schema</h3>

<details>

#### `GET` `/api/schema`

Return a JSON object containing the schema used by the server or return a 404 if no schema file is in use.


*Example*

```bash
curl -X GET 'http://localhost:8000/api/schema'
```

</details>

---

<h3 align='center'>Authentication</h3>

<details>

#### `GET` `/api/auth`

Returns a JSON object containing the servers auth permissions as defined by the default
auth rules or the custom JSON auth as defined in the `Custom Authentication` section
of this guide

*Example*

```bash
curl -X GET 'http://localhost:8000/api/auth'
```

</details>

---

<h3 align='center'>Vector Tiles</h3>

<details>

<p align=right><strong>Admin Only</strong></p>

#### `DELETE` `/api/tiles`

Remove all tiles from the integrated tile cache

*Example*

```bash
curl -X DELETE 'http://localhost:8000/api/tiles
```

---

#### `GET` `/api/tiles/<z>/<x>/<y>`

Request a vector tile for a given set of coordinates. A [Mapbox Vector Tile](https://www.mapbox.com/vector-tiles/) is returned.

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<z>` | `REQUIRED` Desired zoom level for tile
| `<x>` | `REQUIRED` Desired x coordinate for tile
| `<y>` | `REQUIRED` Desired y coordinate for tle

*Example*

```bash
curl -X GET 'http://localhost:8000/api/tiles/1/1/1'
```

---

#### `GET` `/api/tiles/<z>/<x>/<y>/meta`

Return any stored metadata about a given tile.

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<z>` | `REQUIRED` Desired zoom level for tile
| `<x>` | `REQUIRED` Desired x coordinate for tile
| `<y>` | `REQUIRED` Desired y coordinate for tle

*Example*

```bash
curl -X GET 'http://localhost:8000/api/tiles/1/1/1/meta'
```

---

#### `GET` `/api/tiles/<z>/<x>/<y>/regen`

Allows an authenticated user to request a new tile for the given tile coordinates,
ensuring the tile isn't returned from the tile cache.

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<z>` | `REQUIRED` Desired zoom level for tile
| `<x>` | `REQUIRED` Desired x coordinate for tile
| `<y>` | `REQUIRED` Desired y coordinate for tle

*Example*

```bash
curl -X GET \
    -u 'username:password' \
    'http://localhost:8000/api/tiles/1/1/1/regen
```

</details>

---

<h3 align='center'>Webhooks</h3>

<details>

#### `GET` `/api/webhooks`

Return a JSON object containing a list of all webhooks maintained by the server


*Example*

```bash
curl -X GET 'http://localhost:8000/api/webhooks'
```

</details>

---

#### `GET` `/api/webhooks/<id>`

Return a JSON object containing information about a specific webhook

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<id>` | `REQUIRED` ID of the webhook to retrieve


*Example*

```bash
curl -X GET 'http://localhost:8000/api/webhooks/1'
```
---

#### `POST` `/api/webhooks/<id>`

Update a webhook ID with the given webhook data

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<id>` | `REQUIRED` ID of the webhook to update


*Example*

```bash
curl
    -X POST
    -H "Content-Type: application/json" \
    -d '{ "name": "webhook name", "url": "https://example.com", "actions": ["meta", "user", "delta", "style"] }' \
    -u 'username:password' \
    'http://localhost:8000/api/webhooks'
```

#### `DELETE` `/api/webhooks/<id>`

Delete a given webhook

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<id>` | `REQUIRED` ID of the webhook to delete

*Example*

```bash
curl -X DELETE 'http://localhost:8000/api/webhooks/1'
```
---

</details>

---

<h3 align='center'>User Options</h3>

<details>

#### `GET` `/api/users`

Get a list of users (up to 100) or filter by a given user prefix.

*Options*

| Option     | Notes |
| :--------: | ----- |
| `filter` | `Optional` Desired search prefix for username              |
| `limit`  | `Optional` Optionally limit the number of returned results |

*Example*

```bash
curl -X GET 'http://localhost:8000/api/users'
```

---

#### `GET` `/api/user/create`

Create a new user, provied the username & email are not already taken

*Options*

| Option     | Notes |
| :--------: | ----- |
| `username` | `REQUIRED` Desired username, must be unique |
| `password` | `REQUIRED` Desired password |
| `email`    | `REQUIRED` Desired email, must be unique |

*Example*

```bash
curl -X GET 'http://localhost:8000/api/user/create?username=ingalls&password=yeaheh&email=ingalls@protonmail.com
```

---

#### `GET` `/api/user/session`

Return a new session cookie and the `uid` given an Basic Authenticated request.

*Example*

```bash
curl -X GET \
    -u 'username:password' \
    'http://localhost:8000/api/user/session
```

---

#### `GET` `/api/user/info`

Allows an authenticated user to obtain information about their own account

*Example*

```bash
curl -X GET \
    -u 'username:password' \
    'http://localhost:8000/api/user/info'
```

---
<p align=right><strong>Admin Only</strong></p>

#### `GET` `/api/user/<id>`

Obtain information about any user in the system by their numeric User ID.

Note the information returned is the same information that a user is able to
lookup about themself with the `GET /api/user/info` endpoint.

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<id>`     | `REQUIRED` User ID to obtain user information of |

*Example*

```bash
curl -X GET 'http://localhost:8000/api/user/create?username=ingalls&password=yeaheh&email=ingalls@protonmail.com
```

---
<p align=right><strong>Admin Only</strong></p>

#### `PUT` `/api/user/<id>/admin`

Allows an admin to add another user to the admin pool.

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<id>`     | `REQUIRED` User ID to obtain user information of |

*Example*

```bash
curl -X PUT \
    -u 'username:password' \
    'http://localhost:8000/api/user/1/admin'
```

---
<p align=right><strong>Admin Only</strong></p>

#### `DELETE` `/api/user/<id>/admin`

Allows an existing admin to remove another user from the admin pool.

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<id>`     | `REQUIRED` User ID to obtain user information of |

*Example*

```bash
curl -X DELETE \
    -u 'username:password' \
    'http://localhost:8000/api/user/1/admin'
```

</details>

---

<h3 align='center'>Downloading via Clone</h3>

<details>

#### `GET` `/api/data/clone`

Return a Line-Delimited GeoJSON stream of all features currently stored on the server.

Note: All streaming GeoJSON endpoints will send the Unitcode End Of Transmission, EOT
(`0x04`) on stream completion. This can be used to ensure that a stream did not exit early.

*Example*

```bash
curl -X GET 'http://localhost:8000/api/data/clone'
```

</details>

---

<h3 align='center'>Downloading via Query</h3>

<details>

#### `GET` `/api/data/query`

Return a Line-Delimited GeoJSON stream of all features that match the given query.

The query must be a valid SQL query against the `geo` table. Note that the `geo` is
the only table that this endpoint can access. Only read operations are permitted.

Note: All streaming GeoJSON endpoints will send the Unitcode End Of Transmission, EOT
(`0x04`) on stream completion. This can be used to ensure that a stream did not exit early.

IE:

```SQL
SELECT count(*) FROM geo
```

```SQL
SELECT props FROM geo WHERE id = 1
```

*Options*

| Option          | Notes                                                        |
| :-------------: | ------------------------------------------------------------ |
| `query=<query>` | SQL Query to run against Geometries                          |
| `limit=<limit>` | `Optional` Optionally limit the number of returned results   |

*Examples*

```bash
curl -X GET 'http://localhost:8000/api/data/query?query=SELECT%20count(*)%20FROM%20geo'
```

```bash
curl -X GET 'http://localhost:8000/api/data/query?query=SELECT%20props%20FROM%20geo%20WHERE%20id%20%3D%201'
```

</details>

---

<h3 align='center'>Boundaries</h3>

<details>

Boundaries allow downloading data via a set of pre-determined boundary files.

#### `GET` `/api/data/bounds`

Return an array of possible boundary files with which data can be extracted from the server with


*Options*

| Option     | Notes |
| :--------: | ----- |
| `filter` | `Optional` Desired search prefix for username              |
| `limit`  | `Optional` Optionally limit the number of returned results |

*Example*

```bash
curl -X GET 'http://localhost:8000/api/data/bounds'
```

---

#### `GET` `/api/data/bounds/<bounds>`

Return line delimited GeoJSON `Feature` of all the geometries within the specified boundary file.

Note: All streaming GeoJSON endpoints will send the Unitcode End Of Transmission, EOT
(`0x04`) on stream completion. This can be used to ensure that a stream did not exit early.

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<bounds>` | `REQUIRED` One of the boundary files as specified via the `/ap/data/bounds` |

*Example*

```bash
curl -X GET 'http://localhost:8000/api/data/bounds/us_dc'
```

---

#### `POST` `/api/data/bounds/<bounds>`

Create or replace a boundary with the given name.

Note: Boundaries must be a `Polygon` or `MultiPolygon` Feature GeoJSON.

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<bounds>` | `REQUIRED` the name of the bounds to create or replace |

*Example*

```bash
curl -X POST \
    -H "Content-Type: application/json" \
    -d '{"type": "Feature", "properties": {}, "geometry": { "type": "Point", "coordinates": [ 1.1, 1.1 ] } }' \
    -u 'username:password' \
    'http://localhost:8000/api/data/bounds/us_dc'
```

---

#### `DELETE` `/api/data/bounds/<bounds>`

Delete a bounds file with the given name.

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<bounds>` | `REQUIRED` the name of the bounds to create or replace |

*Example*

```bash
curl -X DELETE \
    -u 'username:password' \
    'http://localhost:8000/api/data/bounds/us_dc'
```

---

#### `GET` `/api/data/bounds/<bounds>/stats`

Return statistics about geometries that intersect a given bounds

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<bounds>` | `REQUIRED` One of the boundary files as specified via the `/ap/data/bounds` |

*Example*

```bash
curl -X GET 'http://localhost:8000/api/data/bounds/us_dc/stats'
```

---

#### `GET` `/api/data/bounds/<bounds>/meta`

Return GeoJSON feature representing the bound

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<bounds>` | `REQUIRED` One of the boundary files as specified via the `/ap/data/bounds` |

*Example*

```bash
curl -X GET 'http://localhost:8000/api/data/bounds/us_dc/meta'
```

</details>

---

<h3 align='center'>Downloading Individual Features</h3>

<details>

#### `GET` `/api/data/feature`

Return a single GeoJSON `Feature` given a query parameter

*Options*

| Option            | Notes                                                     |
| :----:            | --------------------------------------------------------- |
| `key=<key>`       | `Optional` Key value to retrieve a given feature by       |
| `point=<lng,lat>` | `Optional` Query for a single feature at the given point  |

*Example*

```bash
curl -X GET 'http://localhost:8000/api/data/feature?key=123'
```

```bash
curl -X GET 'http://localhost:8000/api/data/feature?point=1.1324%2C-45.322'
```

---

#### `GET` `/api/data/feature/<id>`

Return a single GeoJSON `Feature` given its' ID.

*Options*

| Option | Notes |
| :----: | ----- |
| `<id>` | `REQUIRED` Numeric ID of a given feature to download |

*Example*

```bash
curl -X GET 'http://localhost:8000/api/data/feature/1542'
```

---

#### `GET` `/api/data/feature/<id>/history`

Return an array containing the full feature history for the provided feature id.

*Options*

| Option | Notes |
| :----: | ----- |
| `<id>` | `REQUIRED` Numeric ID of a given feature to download |

*Example*

```bash
curl -X GET 'http://localhost:8000/api/data/feature/1542/history'
```

</details>

---

<h3 align='center'>Downloading Multiple Features</h3>

<details>

#### `GET` `/api/data/features`

Return streaming Line-Delimited GeoJSON within the provided BBOX or Point

Note: All streaming GeoJSON endpoints will send the Unitcode End Of Transmission, EOT
(`0x04`) on stream completion. This can be used to ensure that a stream did not exit early.


*Options*

| Option | Notes |
| :----: | ----- |
| `bbox=<minX,minY,maxX,maxY>` | `Optional` Bounding Box in format `left,bottom,right,top` |
| `point=<Lng,Lat>` | `Optional` Point to query for intersections |

*Example*

```bash
curl -X GET 'http://localhost:8000/api/data/features/?bbox=-122.51791%2C37.60447%2C-122.35499%2C37.83244'
```

```bash
curl -X GET 'http://localhost:8000/api/data/features/?point=-95.2734375%2C36.03133177633187'
```

</details>

---

<h3 align='center'>Feature Creation</h3>

<details>

#### `POST` `/api/data/feature` *Auth Required*

Create, Modify, or Delete an individual GeoJSON `Feature`

The Feature must follow format defined in [Feature Format](#feature-format).

The feature also must contain a top-level String `message` attribute describing the changes being made (The delta message)

*Example*

```bash
curl \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"action": "create", "message": "Random Changes", "type":"Feature","properties":{"shop": true},"geometry":{"type":"Point","coordinates":[0,0]}}' \
    -u 'username:password' \
    'http://localhost:8000/api/data/feature'
```

---

#### `POST` `/api/data/features` *Auth Required*

Create, Modify, and/or Delete many features via a GeoJSON `FeatureCollection`

The Features in the FeatureCollection must follow format defined in [Feature Format](#feature-format).

The FeatureCollection also must contain a top-level String `message` attribute describing the changes being made (The delta message)

Note that a mix of `create`, `modify`, and `delete` operations are allowed
within each `FeatureCollection`

*Example*

```bash
curl \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"type":"FeatureCollection","message":"A bunch of changes","features": [{"action": "create", "type":"Feature","properties":{"shop": true},"geometry":{"type":"Point","coordinates":[0,0]}}]}' \
    -u 'username:password' \
    'http://localhost:8000/api/data/features'
```

</details>

---

<h3 align='center'>Deltas</h3>

<details>

#### `GET` `/api/deltas`

Returns an array of the last `limit` defined number of deltas (default: 20). with their corresponding metadata. Does not include geometric
data on the delta. Request a specific delta to get geometric data.

The deltas endpoint has 2 modes. The first is a fixed list of the last `n` deltas. The second is listing deltas by time stamp. the query parameters
for these two modes are mutually exclusive.

*Limit Options*

Return the last `n` deltas starting at the specified `offset`.

Where `n` defaults to 20 and can be up to 100 by utilizing the `limit` parameter

| Option              | Notes |
| :-----------------: | ----- |
| `offset=<delta id>` | Returns the last `n` deltas before the given delta id |
| `limit=<limit>`     | `OPTIONAL` Increase or decrease the max number of returned deltas (Max 100) |

*Date Options*

Return deltas between a given `start` and `end` parameter.

The `start` parameter should be the most recent TIMESTAMP, while the `end` parameter
should be the furthest back in time.

IE: `start` > `end`.

```
   |---------|------|
Current    start   end
 Time
```

- If both `start` and `end` are specified, return all deltas by default
- If `start` or `end` is specified, return last 20 deltas or the number specified by `limit`

| Option     | Notes |
| :--------: | ----- |
| `start`    | `OPTIONAL` Return deltas after n time - ISO 8601 compatible timestamp |
| `end`      | `OPTIONAL` Return deltas before n time - ISO 8601 compatible timestamp |
| `limit`    | `OPTIONAL`  Increase or decrease the max number of returned deltas (Max 100) |

*Example*

```bash
curl -X GET 'http://localhost:8000/api/deltas'
```

```bash
curl -X GET 'http://localhost:8000/api/deltas?offset=3'
```

```bash
curl -X GET 'http://localhost:8000/api/deltas?offset=3&limit=100'
```

---

#### `GET` `/api/deltas/<id>`

Returns all data for a given delta as a JSON Object, including geometric data.

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<id>` | `REQUIRED` Get all data on a given delta

*Example*

```bash
curl -X GET 'http://localhost:8000/api/delta/4'
```

</details>

---

<h3 align='center'>OpenStreetMap API</h3>

The primary goal of the hecate project is a very fast GeoJSON based Interchange. That said, the tooling the OSM community has built around editing is unparalleled. As such,
Hecate provides a Work-In-Progress OpenStreetMap Shim to support a subset of API operations as defined by the [OSM API v0.6](httpl://wiki.openstreetmap.org/wiki/API_v0.6) document.

<details>

*Important Notes*
- All GeoJSON types can be downloaded via the API and viewed in JOSM
- MultiPoints
    - Are represented using an OSM  `Relation`
    - The type will be `multipoint`
    - The member type will be `point`
- MultiLineStrings
    - Are represented using an OSM `Relation`
    - The type will be `multilinestring`
    - The member will be `line`
- Uploading `Way` & `Relation` types are not currently supported, attempting to upload them may produce undesirable results.

The following incomplete list of endpoints are implemented with some degree of coverage with the OSM API Spec but are likely incomplete/or written with the minimum flexibility required to
support editing from JOSM. See the code for a full list.

#### `GET` `/api/capabilities`
#### `GET` `/api/0.6/capabilities`

Return a static XML document describing the capabilities of the API.

*Example*

```bash
curl -X GET 'http://localhost:8000/api/capabilities'
```

---

#### `GET` `/api/0.6/user/details` *Auth Required*

Returns a static XML document describing the number of unread messages that a user has. Every n minutes JOSM checks
this and displays in the interface if there is a new message, to cut down on errors it simply returns a 0 message response.

*Example*

```bash
curl -X GET 'http://localhost:8000/api/0.6/user/details'
```

---

#### `PUT` `/api/0.6/changeset/create` *Auth Required*

Create a new changeset and set the meta information, returning the opened id.

*Example*

```bash
curl \
    -X PUT \
    -d '<osm><changeset><tag k="comment" v="Just adding some streetnames"/></changeset></osm>' \
    'http://localhost:8000/api/0.6/changeset/create
```

---

#### `GET` `/api/0.6/changeset/<changeset_id>/upload` *Auth Required*

Upload osm xml data to a given changeset

*Example*

```bash
curl \
    -X POST \
    -d '<diffResult version="0.6">NODE/WAY/RELATIONS here</diffResult>' \
    'http://localhost:8000/api/0.6/changeset/1/upload'
```

---

#### `PUT` `/api/0.6/changeset/<changeset_id>/close` *Auth Required*

Close a given changeset, preventing further modification to it

*Example*

```bash
curl -X PUT 'http://localhost:8000/api/0.6/changeset/1/close'
```

</details>

---
