//! WASM Filter ABI — Application Binary Interface для фильтров

/// ABI для WASM фильтров
///
/// Фильтры должны экспортировать функцию:
/// `fn filter(input_ptr: i32, input_len: i32) -> i32`
///
/// Возвращает указатель на результат (encoding: [len: 4 bytes][data: len bytes])
pub struct FilterABI;

impl FilterABI {
    /// Имя экспортируемой функции
    pub const FILTER_FUNC_NAME: &'static str = "filter";

    /// Имя функции аллокации памяти
    pub const ALLOC_FUNC_NAME: &'static str = "alloc";

    /// Имя функции освобождения памяти
    pub const FREE_FUNC_NAME: &'static str = "free";
}

/// Маркер для типа фильтра
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    Request,
    Response,
}
