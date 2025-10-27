use axum::{
    extract::Request,
    http::{header, StatusCode, Method},
    middleware::Next,
    response::{Response, Json},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

/// Claims del JWT
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,      // user_id como String
    pub email: String,
    pub exp: usize,       // Expiración (timestamp)
}

/// Extension para inyectar el user_id autenticado en los handlers
#[derive(Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
}

/// Middleware de autenticación JWT PERMISIVO con debugging detallado
pub async fn auth_middleware(
    mut req: Request,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();

    tracing::debug!("🔍 [AUTH MIDDLEWARE] Iniciando - {} {}", method, path);

    // Intentar extraer token, pero NO fallar si no existe
    if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
        tracing::debug!("📨 [AUTH MIDDLEWARE] Header Authorization encontrado");

        if let Ok(auth_str) = auth_header.to_str() {
            tracing::debug!("🔐 [AUTH MIDDLEWARE] Header válido: {}...", &auth_str[..std::cmp::min(20, auth_str.len())]);

            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                tracing::debug!("✅ [AUTH MIDDLEWARE] Formato 'Bearer' correcto");
                tracing::debug!("🎫 [AUTH MIDDLEWARE] Token: {}...", &token[..std::cmp::min(10, token.len())]);

                let jwt_secret = std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "fallback-secret-key".to_string());

                // Intentar decodificar el token
                match decode::<Claims>(
                    token,
                    &DecodingKey::from_secret(jwt_secret.as_bytes()),
                    &Validation::default(),
                ) {
                    Ok(token_data) => {
                        // 👈 GUARDAR LOS VALORES ANTES DE MOVERLOS
                        let user_id_str = token_data.claims.sub.clone();  // Clonar para el log
                        let email = token_data.claims.email.clone();      // Clonar para el log

                        tracing::debug!("🔑 [AUTH MIDDLEWARE] Token decodificado exitosamente");
                        tracing::debug!("📝 [AUTH MIDDLEWARE] Claims: sub={}, email={}",
                                      user_id_str, email);

                        // Parsear el user_id desde el claim "sub"
                        match Uuid::parse_str(&user_id_str) {
                            Ok(user_id) => {
                                // Token válido - agregar user a extensions
                                let auth_user = AuthUser {
                                    user_id,
                                    email: email.clone(),  // Usar el clon
                                };
                                req.extensions_mut().insert(auth_user);

                                tracing::info!("✅ [AUTH MIDDLEWARE] {} {} - Usuario autenticado: {} ({})",
                                             method, path, user_id, email);  // ✅ Usar el clon
                            }
                            Err(_) => {
                                tracing::warn!("❌ [AUTH MIDDLEWARE] {} {} - Token con user_id inválido: {}",
                                             method, path, user_id_str);  // ✅ Usar el clon
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("❌ [AUTH MIDDLEWARE] {} {} - Token inválido: {}",
                                     method, path, e);
                    }
                }
            } else {
                tracing::warn!("⚠️ [AUTH MIDDLEWARE] {} {} - Formato de token inválido (falta 'Bearer ' prefix). Header: {}...",
                             method, path, &auth_str[..std::cmp::min(30, auth_str.len())]);
            }
        } else {
            tracing::warn!("⚠️ [AUTH MIDDLEWARE] {} {} - Header Authorization no es UTF-8 válido",
                         method, path);
        }
    } else {
        // Identificar si es una ruta pública esperada o un intento no autorizado
        let public_routes = [
            "/auth/register",
            "/auth/login",
            "/health",
            "/health/db",
            "/health/tigerbeetle"
        ];

        let is_public_route = public_routes.iter().any(|&route| path.starts_with(route));

        if is_public_route {
            tracing::debug!("👋 [AUTH MIDDLEWARE] {} {} - Ruta pública (sin token esperado)",
                          method, path);
        } else {
            tracing::warn!("🚫 [AUTH MIDDLEWARE] {} {} - Intento de acceso SIN token a ruta potencialmente protegida",
                         method, path);
        }
    }

    // Verificar si se agregó un usuario autenticado
    let has_auth_user = req.extensions().get::<AuthUser>().is_some();
    if has_auth_user {
        tracing::debug!("👍 [AUTH MIDDLEWARE] {} {} - Request continúa CON usuario autenticado",
                      method, path);
    } else {
        tracing::debug!("👤 [AUTH MIDDLEWARE] {} {} - Request continúa SIN usuario autenticado",
                      method, path);
    }

    // 👈 SIEMPRE continuamos, incluso sin token o con token inválido
    let response = next.run(req).await;

    // Log adicional después de procesar la request
    let status = response.status();
    tracing::debug!("📤 [AUTH MIDDLEWARE] {} {} -> {} (auth: {})",
                  method, path, status, if has_auth_user { "SI" } else { "NO" });

    response
}