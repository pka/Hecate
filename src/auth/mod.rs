use rocket::request::{self, FromRequest};
use rocket::http::Status;
use rocket::{Request, Outcome};

use crate::err::HecateError;

fn not_authed() -> HecateError {
    HecateError::new(401, String::from("You must be logged in to access this resource"), None)
}

///
/// Allows a category to be null, public, admin, or user
///
/// This category makes up the majority of endpoints in hecate and is the most
/// flexible
///
fn is_all(scope_type: &str, scope: &Option<String>) -> Result<bool, String> {
    match scope {
        &None => Ok(true),
        &Some(ref scope_str) => {
            match scope_str as &str {
                "public" => Ok(true),
                "admin" => Ok(true),
                "user" => Ok(true),
                _ => Err(format!("Auth Config Error: '{}' must be one of 'public', 'admin', 'user', or null", scope_type)),
            }
        }
    }
}

///
/// Allows a category to be null, self, or admin
///
/// This category is used for CRUD operations against data for a specfic user,
/// not only must the user be logged in but the user can only update their own
/// data
///
fn is_self(scope_type: &str, scope: &Option<String>) -> Result<bool, String> {
    match scope {
        &None => Ok(true),
        &Some(ref scope_str) => {
            match scope_str as &str {
                "self" => Ok(true),
                "admin" => Ok(true),
                _ => Err(format!("Auth Config Error: '{}' must be one of 'self', 'admin', or null", scope_type)),
            }
        }
    }
}

///
/// Allows a category to be null, user, or admin
///
/// This category is used primarily for feature operations. The user must be
/// logged in but can make changes to any feature, including features created
/// by another user
///
fn is_auth(scope_type: &str, scope: &Option<String>) -> Result<bool, String> {
    match scope {
        &None => Ok(true),
        &Some(ref scope_str) => {
            match scope_str as &str {
                "user" => Ok(true),
                "admin" => Ok(true),
                _ => Err(format!("Auth Config Error: '{}' must be one of 'self', 'admin', or null", scope_type)),
            }
        }
    }
}

pub trait ValidAuth {
    fn is_valid(&self) -> Result<bool, String>;
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AuthWebhooks {
    pub list: Option<String>,
    pub delete: Option<String>,
    pub update: Option<String>
}

impl AuthWebhooks {
    fn new() -> Self {
        AuthWebhooks {
            list: Some(String::from("admin")),
            delete: Some(String::from("admin")),
            update: Some(String::from("admin"))
        }
    }
}

impl ValidAuth for AuthWebhooks {
    fn is_valid(&self) -> Result<bool, String> {
        is_auth("webhooks::list", &self.list)?;
        is_auth("webhooks::delete", &self.delete)?;
        is_auth("webhooks::update", &self.update)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AuthMeta {
    pub get: Option<String>,
    pub list: Option<String>,
    pub set: Option<String>
}

impl AuthMeta {
    fn new() -> Self {
        AuthMeta {
            get: Some(String::from("public")),
            list: Some(String::from("public")),
            set: Some(String::from("admin"))
        }
    }
}

impl ValidAuth for AuthMeta {
    fn is_valid(&self) -> Result<bool, String> {
        is_all("meta::get", &self.get)?;
        is_all("meta::list", &self.list)?;
        is_auth("meta::set", &self.set)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AuthClone {
    pub get: Option<String>,
    pub query: Option<String>
}

impl AuthClone {
    fn new() -> Self {
        AuthClone {
            get: Some(String::from("user")),
            query: Some(String::from("user"))
        }
    }
}

impl ValidAuth for AuthClone {
    fn is_valid(&self) -> Result<bool, String> {
        is_all("clone::get", &self.get)?;
        is_all("clone::query", &self.query)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AuthSchema {
    pub get: Option<String>
}

impl AuthSchema {
    fn new() -> Self {
        AuthSchema {
            get: Some(String::from("public"))
        }
    }
}

impl ValidAuth for AuthSchema {
    fn is_valid(&self) -> Result<bool, String> {
        is_all("schema::get", &self.get)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AuthStats {
    pub get: Option<String>,
    pub bounds: Option<String>
}

impl AuthStats {
    fn new() -> Self {
        AuthStats {
            get: Some(String::from("public")),
            bounds: Some(String::from("public"))
        }
    }
}

impl ValidAuth for AuthStats {
    fn is_valid(&self) -> Result<bool, String> {
        is_all("stats::get", &self.get)?;
        is_all("stats::bounds", &self.bounds)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AuthAuth {
    pub get: Option<String>
}

impl AuthAuth {
    fn new() -> Self {
        AuthAuth {
            get: Some(String::from("public"))
        }
    }
}

impl ValidAuth for AuthAuth {
    fn is_valid(&self) -> Result<bool, String> {
        is_all("auth::get", &self.get)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AuthMVT {
    pub get: Option<String>,
    pub delete: Option<String>,
    pub regen: Option<String>,
    pub meta: Option<String>
}

impl AuthMVT {
    fn new() -> Self {
        AuthMVT {
            get: Some(String::from("public")),
            delete: Some(String::from("admin")),
            regen: Some(String::from("user")),
            meta: Some(String::from("public"))
        }
    }
}

impl ValidAuth for AuthMVT {
    fn is_valid(&self) -> Result<bool, String> {
        is_all("mvt::get", &self.get)?;
        is_all("mvt::regen", &self.regen)?;
        is_all("mvt::delete", &self.regen)?;
        is_all("mvt::meta", &self.meta)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AuthUser {
    pub info: Option<String>,
    pub list: Option<String>,
    pub create: Option<String>,
    pub create_session: Option<String>
}

impl AuthUser {
    fn new() -> Self {
        AuthUser {
            info: Some(String::from("self")),
            list: Some(String::from("user")),
            create: Some(String::from("public")),
            create_session: Some(String::from("self"))
        }
    }
}

impl ValidAuth for AuthUser {
    fn is_valid(&self) -> Result<bool, String> {
        is_all("user::create", &self.create)?;
        is_all("user::list", &self.list)?;

        is_self("user::create_session", &self.create_session)?;
        is_self("user::info", &self.info)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AuthStyle {
    pub create: Option<String>,
    pub patch: Option<String>,
    pub set_public: Option<String>,
    pub set_private: Option<String>,
    pub delete: Option<String>,
    pub get: Option<String>,
    pub list: Option<String>
}

impl AuthStyle {
    fn new() -> Self {
        AuthStyle {
            create: Some(String::from("self")),
            patch: Some(String::from("self")),
            set_public: Some(String::from("self")),
            set_private: Some(String::from("self")),
            delete: Some(String::from("self")),
            get: Some(String::from("public")),
            list: Some(String::from("public"))
        }
    }
}

impl ValidAuth for AuthStyle {
    fn is_valid(&self) -> Result<bool, String> {
        is_self("style::create", &self.create)?;
        is_self("style::patch", &self.patch)?;
        is_self("style::set_public", &self.set_public)?;
        is_self("style::set_private", &self.set_private)?;
        is_self("style::delete", &self.delete)?;
        is_all("style::get", &self.get)?;
        is_all("style::list", &self.list)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AuthDelta {
    pub get: Option<String>,
    pub list: Option<String>,
}

impl AuthDelta {
    fn new() -> Self {
        AuthDelta {
            get: Some(String::from("public")),
            list: Some(String::from("public"))
        }
    }
}

impl ValidAuth for AuthDelta {
    fn is_valid(&self) -> Result<bool, String> {
        is_all("delta::get", &self.get)?;
        is_all("delta::list", &self.list)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AuthFeature {
    pub force: Option<String>,
    pub create: Option<String>,
    pub get: Option<String>,
    pub history: Option<String>
}

impl AuthFeature {
    fn new() -> Self {
        AuthFeature {
            force: Some(String::from("none")),
            create: Some(String::from("user")),
            get: Some(String::from("public")),
            history: Some(String::from("public"))
        }
    }
}

impl ValidAuth for AuthFeature {
    fn is_valid(&self) -> Result<bool, String> {
        is_auth("feature::create", &self.create)?;
        is_auth("feature::force", &self.force)?;
        is_all("feature::get", &self.get)?;
        is_all("feature::history", &self.history)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AuthBounds {
    pub list: Option<String>,
    pub create: Option<String>,
    pub delete: Option<String>,
    pub get: Option<String>
}

impl AuthBounds {
    fn new() -> Self {
        AuthBounds {
            list: Some(String::from("public")),
            create: Some(String::from("admin")),
            delete: Some(String::from("admin")),
            get: Some(String::from("public"))
        }
    }
}

impl ValidAuth for AuthBounds {
    fn is_valid(&self) -> Result<bool, String> {
        is_all("bounds::list", &self.list)?;
        is_all("bounds::create", &self.create)?;
        is_all("bounds::delete", &self.create)?;
        is_all("bounds::get", &self.get)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AuthOSM {
    pub get: Option<String>,
    pub create: Option<String>
}

impl AuthOSM {
    fn new() -> Self {
        AuthOSM {
            get: Some(String::from("public")),
            create: Some(String::from("user"))
        }
    }
}

impl ValidAuth for AuthOSM {
    fn is_valid(&self) -> Result<bool, String> {
        is_all("osm::get", &self.get)?;
        is_auth("osm::create", &self.create)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CustomAuth {
    pub server: Option<String>,
    pub meta: Option<AuthMeta>,
    pub webhooks: Option<AuthWebhooks>,
    pub stats: Option<AuthStats>,
    pub mvt: Option<AuthMVT>,
    pub schema: Option<AuthSchema>,
    pub auth: Option<AuthAuth>,
    pub user: Option<AuthUser>,
    pub feature: Option<AuthFeature>,
    pub style: Option<AuthStyle>,
    pub delta: Option<AuthDelta>,
    pub bounds: Option<AuthBounds>,
    pub clone: Option<AuthClone>,
    pub osm: Option<AuthOSM>
}

impl ValidAuth for CustomAuth {
    fn is_valid(&self) -> Result<bool, String> {
        is_all("server", &self.server)?;

        match &self.meta {
            None => (),
            Some(ref meta) => { meta.is_valid()?; }
        };

        match &self.mvt {
            None => (),
            Some(ref mvt) => { mvt.is_valid()?; }
        };

        match &self.schema {
            None => (),
            Some(ref schema) => { schema.is_valid()?; }
        };

        match &self.user {
            None => (),
            Some(ref user) => { user.is_valid()?; }
        };

        match &self.feature {
            None => (),
            Some(ref feature) => { feature.is_valid()?; }
        };

        match &self.style {
            None => (),
            Some(ref style) => { style.is_valid()?; }
        };

        match &self.delta {
            None => (),
            Some(ref delta) => { delta.is_valid()?; }
        };

        match &self.bounds {
            None => (),
            Some(ref bounds) => { bounds.is_valid()?; }
        };

        match &self.clone {
            None => (),
            Some(ref clone) => { clone.is_valid()?; }
        };

        match &self.osm {
            None => (),
            Some(ref osm) => { osm.is_valid()?; }
        };

        Ok(true)
    }
}

///
/// Determines whether the current auth state meets or exceeds the
/// requirements of an endpoint
///
fn auth_met(required: &Option<String>, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
    auth.validate(conn)?;

    match required {
        None => Err(not_authed()),
        Some(req) => match req.as_ref() {
            "public" => Ok(true),
            "admin" => {
                if auth.uid.is_none() || auth.access.is_none() {
                    return Err(not_authed());
                } else if auth.access == Some(String::from("admin")) {
                    return Ok(true);
                } else {
                    return Err(not_authed());
                }
            },
            "user" => {
                if auth.uid.is_some() {
                    return Ok(true);
                } else {
                    return Err(not_authed());
                }
            },
            "self" => {
                //Note: This ensures the user is validated,
                //it is up to the parent caller to ensure
                //the UID of 'self' matches the requested resource

                if auth.uid.is_some() {
                    return Ok(true);
                } else {
                    return Err(not_authed());
                }
            },
            _ => Err(not_authed())
        }
    }
}

impl CustomAuth {
    pub fn new() -> Self {
        CustomAuth {
            server: Some(String::from("public")),
            webhooks: Some(AuthWebhooks::new()),
            meta: Some(AuthMeta::new()),
            stats: Some(AuthStats::new()),
            schema: Some(AuthSchema::new()),
            auth: Some(AuthAuth::new()),
            mvt: Some(AuthMVT::new()),
            user: Some(AuthUser::new()),
            feature: Some(AuthFeature::new()),
            style: Some(AuthStyle::new()),
            delta: Some(AuthDelta::new()),
            bounds: Some(AuthBounds::new()),
            clone: Some(AuthClone::new()),
            osm: Some(AuthOSM::new())
        }
    }

    pub fn to_json(&self) -> serde_json::value::Value {
        let json_auth = serde_json::from_str(serde_json::to_string(&self).unwrap().as_str()).unwrap();

        json_auth
    }


    pub fn is_admin(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        auth_met(&Some(String::from("admin")), auth, conn)
    }

    pub fn allows_server(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        auth_met(&self.server, auth, conn)
    }

    pub fn allows_webhooks_list(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.webhooks {
            None => Err(not_authed()),
            Some(webhooks) => auth_met(&webhooks.list, auth, conn)
        }
    }

    pub fn allows_webhooks_delete(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.webhooks {
            None => Err(not_authed()),
            Some(webhooks) => auth_met(&webhooks.delete, auth, conn)
        }
    }

    pub fn allows_webhooks_update(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.webhooks {
            None => Err(not_authed()),
            Some(webhooks) => auth_met(&webhooks.update, auth, conn)
        }
    }

    pub fn allows_meta_get(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.meta {
            None => Err(not_authed()),
            Some(meta) => auth_met(&meta.get, auth, conn)
        }
    }

    pub fn allows_meta_list(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.meta {
            None => Err(not_authed()),
            Some(meta) => auth_met(&meta.list, auth, conn)
        }
    }

    pub fn allows_meta_set(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.meta {
            None => Err(not_authed()),
            Some(meta) => auth_met(&meta.set, auth, conn)
        }
    }

    pub fn allows_stats_get(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.stats {
            None => Err(not_authed()),
            Some(stats) => auth_met(&stats.get, auth, conn)
        }
    }

    pub fn allows_stats_bounds(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.stats {
            None => Err(not_authed()),
            Some(stats) => auth_met(&stats.bounds, auth, conn)
        }
    }

    pub fn allows_mvt_get(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.mvt {
            None => Err(not_authed()),
            Some(mvt) => auth_met(&mvt.get, auth, conn)
        }
    }

    pub fn allows_mvt_delete(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.mvt {
            None => Err(not_authed()),
            Some(mvt) => auth_met(&mvt.delete, auth, conn)
        }
    }

    pub fn allows_mvt_regen(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.mvt {
            None => Err(not_authed()),
            Some(mvt) => auth_met(&mvt.regen, auth, conn)
        }
    }

    pub fn allows_mvt_meta(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.mvt {
            None => Err(not_authed()),
            Some(mvt) => auth_met(&mvt.meta, auth, conn)
        }
    }

    pub fn allows_user_list(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.user {
            None => Err(not_authed()),
            Some(user) => auth_met(&user.list, auth, conn)
        }
    }

    pub fn allows_user_create(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.user {
            None => Err(not_authed()),
            Some(user) => auth_met(&user.create, auth, conn)
        }
    }

    pub fn allows_user_info(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.user {
            None => Err(not_authed()),
            Some(user) => auth_met(&user.info, auth, conn)
        }
    }

    pub fn allows_user_create_session(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.user {
            None => Err(not_authed()),
            Some(user) => auth_met(&user.create_session, auth, conn)
        }
    }

    pub fn allows_style_create(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.style {
            None => Err(not_authed()),
            Some(style) => auth_met(&style.create, auth, conn)
        }
    }

    pub fn allows_style_patch(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.style {
            None => Err(not_authed()),
            Some(style) => auth_met(&style.patch, auth, conn)
        }
    }

    pub fn allows_style_set_public(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.style {
            None => Err(not_authed()),
            Some(style) => auth_met(&style.set_public, auth, conn)
        }
    }

    pub fn allows_style_set_private(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.style {
            None => Err(not_authed()),
            Some(style) => auth_met(&style.set_private, auth, conn)
        }
    }

    pub fn allows_style_delete(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.style {
            None => Err(not_authed()),
            Some(style) => auth_met(&style.delete, auth, conn)
        }
    }

    pub fn allows_style_get(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.style {
            None => Err(not_authed()),
            Some(style) => auth_met(&style.get, auth, conn)
        }
    }

    pub fn allows_style_list(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.style {
            None => Err(not_authed()),
            Some(style) => auth_met(&style.list, auth, conn)
        }
    }

    pub fn allows_delta_get(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.delta {
            None => Err(not_authed()),
            Some(delta) => auth_met(&delta.get, auth, conn)
        }
    }

    pub fn allows_delta_list(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.delta {
            None => Err(not_authed()),
            Some(delta) => auth_met(&delta.list, auth, conn)
        }
    }

    pub fn allows_clone_get(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.clone {
            None => Err(not_authed()),
            Some(clone) => auth_met(&clone.get, auth, conn)
        }
    }

    pub fn allows_clone_query(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.clone {
            None => Err(not_authed()),
            Some(clone) => auth_met(&clone.query, auth, conn)
        }
    }

    pub fn allows_bounds_get(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.bounds {
            None => Err(not_authed()),
            Some(bounds) => auth_met(&bounds.get, auth, conn)
        }
    }

    pub fn allows_bounds_create(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.bounds {
            None => Err(not_authed()),
            Some(bounds) => auth_met(&bounds.create, auth, conn)
        }
    }

    pub fn allows_bounds_delete(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.bounds {
            None => Err(not_authed()),
            Some(bounds) => auth_met(&bounds.delete, auth, conn)
        }
    }

    pub fn allows_bounds_list(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.bounds {
            None => Err(not_authed()),
            Some(bounds) => auth_met(&bounds.list, auth, conn)
        }
    }

    pub fn allows_feature_create(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.feature {
            None => Err(not_authed()),
            Some(feature) => auth_met(&feature.create, auth, conn)
        }
    }

    pub fn allows_feature_force(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.feature {
            None => Err(not_authed()),
            Some(feature) => auth_met(&feature.force, auth, conn)
        }
    }

    pub fn allows_feature_get(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.feature {
            None => Err(not_authed()),
            Some(feature) => auth_met(&feature.get, auth, conn)
        }
    }

    pub fn allows_feature_history(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.feature {
            None => Err(not_authed()),
            Some(feature) => auth_met(&feature.history, auth, conn)
        }
    }

    pub fn allows_schema_get(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.schema {
            None => Err(not_authed()),
            Some(schema) => auth_met(&schema.get, auth, conn)
        }
    }

    pub fn allows_auth_get(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.auth {
            None => Err(not_authed()),
            Some(a) => auth_met(&a.get, auth, conn)
        }
    }

    pub fn allows_osm_get(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.osm {
            None => Err(not_authed()),
            Some(osm) => auth_met(&osm.get, auth, conn)
        }
    }

    pub fn allows_osm_create(&self, auth: &mut Auth, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match &self.osm {
            None => Err(not_authed()),
            Some(osm) => auth_met(&osm.create, auth, conn)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Auth {
    pub uid: Option<i64>,
    pub access: Option<String>,
    pub token: Option<String>,
    pub basic: Option<(String, String)>
}

impl Auth {
    pub fn new() -> Self {
        Auth {
            uid: None,
            access: None,
            token: None,
            basic: None
        }
    }

    ///
    /// Remove user data from the Auth object
    ///
    /// Used as a generic function by validate to ensure future
    /// authentication methods are cleared with each validate
    ///
    pub fn secure(&mut self, user: Option<(i64, Option<String>)>) {
        match user {
            Some(user) => {
                self.uid = Some(user.0);
                self.access = user.1;
            }
            _ => ()
        }
        self.token = None;
        self.basic = None;
    }

    ///
    /// The Rocket Request guard simply provides a utility wrapper from the request to a more
    /// easily parsable auth object. It **does not** perform any authentication.
    ///
    /// This function takes the populated Auth struct and checks if the token/basic auth is valid,
    /// populated the uid field
    ///
    /// Note: Once validated the token/basic auth used to validate the user will be set to null
    ///
    pub fn validate(&mut self, conn: &impl postgres::GenericConnection) -> Result<Option<i64>, HecateError> {
        if self.basic.is_some() {
            match conn.query("
                SELECT
                    id,
                    access
                FROM users
                WHERE
                    username = $1
                    AND password = crypt($2, password)
            ", &[ &self.basic.as_ref().unwrap().0 , &self.basic.as_ref().unwrap().1 ]) {
                Ok(res) => {
                    if res.len() != 1 {
                        return Err(not_authed());
                    }

                    let uid: i64 = res.get(0).get(0);
                    let access: Option<String> = res.get(0).get(1);

                    self.secure(Some((uid, access)));

                    return Ok(Some(uid));
                },
                _ => {
                    return Err(not_authed());
                }
            }
        } else if self.token.is_some() {
            match conn.query("
                SELECT
                    users_tokens.uid,
                    users.access
                FROM
                    users_tokens,
                    users
                WHERE
                    token = $1
                    AND now() < expiry
                    AND users_tokens.uid = users.id
            ", &[ &self.token.as_ref().unwrap() ]) {
                Ok(res) => {
                    if res.len() == 0 {
                        return Err(not_authed());
                    }

                    let uid: i64 = res.get(0).get(0);
                    let access: Option<String> = res.get(0).get(1);

                    self.secure(Some((uid, access)));

                    return Ok(Some(uid));
                },
                _ => {
                    return Err(not_authed());
                }
            }
        }

        Ok(None)
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Auth {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Auth, ()> {
        let mut auth = Auth::new();

        match request.cookies().get("session") {
            Some(token) => {
                auth.token = Some(String::from(token.value()));

                return Outcome::Success(auth);
            },
            None => ()
        };

        let keys: Vec<_> = request.headers().get("Authorization").collect();

        //Auth Failed - None object returned
        if keys.len() != 1 || keys[0].len() < 7 {
            return Outcome::Success(auth);
        }

        let mut authtype = String::from(keys[0]);
        let auth_str = authtype.split_off(6);

        if authtype != "Basic " {
            return Outcome::Success(auth);
        }

        match base64::decode(&auth_str) {
            Ok(decoded) => match String::from_utf8(decoded) {
                Ok(decoded_str) => {

                    let split = decoded_str.split(":").collect::<Vec<&str>>();

                    if split.len() != 2 { return Outcome::Failure((Status::Unauthorized, ())); }

                    auth.basic = Some((String::from(split[0]), String::from(split[1])));

                    Outcome::Success(auth)
                },
                Err(_) => Outcome::Failure((Status::Unauthorized, ()))
            },
            Err(_) => Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}
