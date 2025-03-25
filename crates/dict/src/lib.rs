use anyhow::{Context, Result};
use model::dictionary;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, entity::*, query::*};
use serde::de::DeserializeOwned;
use std::{sync::Arc, time::Duration};
use tracing::log::LevelFilter;

mod code;
pub use code::DictCode;

#[derive(Clone)]
pub struct Dict {
    conn: Arc<DatabaseConnection>,
}

impl Dict {
    pub fn new(conn: Arc<DatabaseConnection>) -> Self {
        Self { conn }
    }

    pub async fn new_from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")?;
        let mut opt = ConnectOptions::new(database_url);
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(5))
            .acquire_timeout(Duration::from_secs(5))
            .sqlx_logging(true)
            .sqlx_logging_level(LevelFilter::Debug);

        let conn = Database::connect(opt).await?;
        Ok(Self::new(Arc::new(conn)))
    }

    /// 获取指定代码的值
    pub async fn get_value(&self, code: impl Into<DictCode>) -> Result<Option<String>> {
        let code = code.into();
        Ok(self
            .find_by_code(&code.to_string())
            .await?
            .map(|model| model.value))
    }

    /// 获取指定代码的值并解析为指定类型
    pub async fn get_value_as<T: DeserializeOwned>(
        &self,
        code: impl Into<DictCode>,
    ) -> Result<Option<T>> {
        let code = code.into();
        let value = self.get_value(code).await?;
        match value {
            Some(value) => {
                let parsed = serde_json::from_str(&value)
                    .with_context(|| format!("解析值失败: {}", value))?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    /// 获取指定分组的所有值
    pub async fn get_group_values(&self, group_code: &str) -> Result<Vec<(String, String)>> {
        let models = self.find_by_group(group_code).await?;
        Ok(models
            .into_iter()
            .map(|model| (model.code, model.value))
            .collect())
    }

    /// 获取指定分组的所有值并解析为指定类型
    pub async fn get_group_values_as<T: DeserializeOwned>(
        &self,
        group_code: &str,
    ) -> Result<Vec<(String, T)>> {
        let models = self.find_by_group(group_code).await?;
        let mut result = Vec::new();

        for model in models {
            let parsed = serde_json::from_str(&model.value)
                .with_context(|| format!("解析值失败: {}", model.value))?;
            result.push((model.code, parsed));
        }

        Ok(result)
    }

    /// 设置值（不存在则创建，存在则更新）
    pub async fn set_value(
        &self,
        code: impl Into<DictCode>,
        value: impl Into<String>,
    ) -> Result<()> {
        let code = code.into();
        let model = dictionary::Model {
            code: code.to_string(),
            group_code: code.group().to_string(),
            value: value.into(),
            sort_order: Some(code.sort_order()),
            description: Some(code.description().to_string()),
        };

        self.upsert_many(vec![model]).await
    }

    /// 设置值（支持序列化）
    pub async fn set_value_as<T: serde::Serialize>(
        &self,
        code: impl Into<DictCode>,
        value: &T,
    ) -> Result<()> {
        let value_str = serde_json::to_string(value).with_context(|| "序列化值失败")?;

        self.set_value(code, value_str).await
    }

    /// 插入新记录
    pub async fn insert(&self, entry: dictionary::Model) -> Result<()> {
        dictionary::Entity::insert(entry.into_active_model())
            .exec(self.conn.as_ref())
            .await?;
        Ok(())
    }

    /// 更新记录
    pub async fn update(&self, entry: dictionary::Model) -> Result<()> {
        dictionary::Entity::update(entry.into_active_model())
            .exec(self.conn.as_ref())
            .await?;
        Ok(())
    }

    /// 删除记录
    pub async fn delete(&self, code: impl Into<DictCode>) -> Result<()> {
        let code = code.into();
        dictionary::Entity::delete_by_id(code.to_string())
            .exec(self.conn.as_ref())
            .await?;
        Ok(())
    }

    /// 根据代码查询记录
    pub async fn find_by_code(&self, code: &str) -> Result<Option<dictionary::Model>> {
        let result = dictionary::Entity::find_by_id(code.to_string())
            .one(self.conn.as_ref())
            .await?;
        Ok(result)
    }

    /// 查询所有记录
    pub async fn find_all(&self) -> Result<Vec<dictionary::Model>> {
        let results = dictionary::Entity::find()
            .order_by_asc(dictionary::Column::SortOrder)
            .all(self.conn.as_ref())
            .await?;
        Ok(results)
    }

    /// 根据分组代码查询记录
    pub async fn find_by_group(&self, group_code: &str) -> Result<Vec<dictionary::Model>> {
        let results = dictionary::Entity::find()
            .filter(dictionary::Column::GroupCode.eq(group_code))
            .order_by_asc(dictionary::Column::SortOrder)
            .all(self.conn.as_ref())
            .await?;
        Ok(results)
    }

    /// 根据分组代码和代码查询记录
    pub async fn find_by_group_and_code(
        &self,
        group_code: &str,
        code: impl Into<DictCode>,
    ) -> Result<Option<dictionary::Model>> {
        let code = code.into();
        let result = dictionary::Entity::find()
            .filter(
                Condition::all()
                    .add(dictionary::Column::GroupCode.eq(group_code))
                    .add(dictionary::Column::Code.eq(code.to_string())),
            )
            .one(self.conn.as_ref())
            .await?;
        Ok(result)
    }

    /// 批量插入或更新记录
    pub async fn upsert_many(&self, entries: Vec<dictionary::Model>) -> Result<()> {
        for entry in entries {
            let active_model = entry.into_active_model();
            dictionary::Entity::insert(active_model)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(dictionary::Column::Code)
                        .update_columns([
                            dictionary::Column::GroupCode,
                            dictionary::Column::Value,
                            dictionary::Column::SortOrder,
                            dictionary::Column::Description,
                        ])
                        .to_owned(),
                )
                .exec(self.conn.as_ref())
                .await?;
        }
        Ok(())
    }

    /// 批量删除记录
    pub async fn delete_by_group(&self, group_code: &str) -> Result<()> {
        dictionary::Entity::delete_many()
            .filter(dictionary::Column::GroupCode.eq(group_code))
            .exec(self.conn.as_ref())
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        value: i32,
    }

    #[tokio::test]
    #[ignore]
    async fn test_dict() -> Result<()> {
        dotenv::dotenv().ok();
        let _ = Dict::new_from_env().await?;
        Ok(())
    }
}
