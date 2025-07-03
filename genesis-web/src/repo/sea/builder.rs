use sea_orm::Value;

pub struct SqlBuilder {
    sql: String,
    params: Vec<Value>,
    has_where: bool, // 标记是否已有 WHERE 子句
}

impl SqlBuilder {
    pub fn new(base_sql: &str) -> Self {
        let has_where = base_sql.to_uppercase().contains("WHERE");
        Self {
            sql: base_sql.to_string(),
            params: Vec::new(),
            has_where,
        }
    }

    /// 添加 LIKE 条件（自动添加通配符）
    pub fn and_like(mut self, column: &str, value: &str) -> Self {
        self.add_condition(column, "LIKE", format!("%{}%", value));
        self
    }

    /// 添加精确匹配条件
    pub fn and_eq<T: Into<Value>>(mut self, column: &str, value: T) -> Self {
        self.add_condition(column, "=", value);
        self
    }

    /// 可选条件（跳过 None）
    pub fn try_filter_like(mut self, column: &str, value: Option<impl AsRef<str>>) -> Self {
        if let Some(v) = value {
            self.add_condition(column, "LIKE", format!("%{}%", v.as_ref()));
        }
        self
    }

    pub fn try_filter_eq<T: Into<Value>>(mut self, column: &str, value: Option<T>) -> Self {
        if let Some(v) = value {
            self.add_condition(column, "=", v.into());
        }
        self
    }

    /// 私有方法：统一处理条件添加
    fn add_condition<T: Into<Value>>(&mut self, column: &str, operator: &str, value: T) {
        let prefix = if self.has_where { "AND" } else { "WHERE" };
        self.sql
            .push_str(&format!(" {} {} {} ?", prefix, column, operator));
        self.params.push(value.into());
        self.has_where = true;
    }

    /// 最终构建
    pub fn build(self) -> (String, Vec<Value>) {
        (self.sql, self.params)
    }
}
