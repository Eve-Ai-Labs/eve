use jwt::JwtSecret;
use poem::{
    http::StatusCode,
    web::headers::{self, authorization::Bearer, HeaderMapExt},
    Endpoint, Error, Middleware, Request, Result,
};
use tracing::debug;

pub struct JwtAuth(pub JwtSecret);

impl<E: Endpoint> Middleware<E> for JwtAuth {
    type Output = JwtAuthEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        JwtAuthEndpoint { ep, jwt: self.0 }
    }
}

pub struct JwtAuthEndpoint<E> {
    ep: E,
    jwt: JwtSecret,
}

impl<E: Endpoint> Endpoint for JwtAuthEndpoint<E> {
    type Output = E::Output;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        if let Some(auth) = req.headers().typed_get::<headers::Authorization<Bearer>>() {
            // @todo check token validity
            let _claim = self.jwt.decode(auth.token()).map_err(|err| {
                debug!("Error: {err}");
                Error::from_string("invalid JWT token", StatusCode::BAD_REQUEST)
            })?;

            self.ep.call(req).await
        } else {
            Err(Error::from_string(
                "A Jwt token is required",
                StatusCode::UNAUTHORIZED,
            ))
        }
    }
}
