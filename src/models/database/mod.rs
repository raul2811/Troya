// src/models/database/mod.rs

// 1. Declara los módulos de tus archivos
pub mod user;
pub mod transaction;

// 2. Re-exporta los structs para un uso fácil
pub use user::User;
pub use transaction::TransactionHistory;