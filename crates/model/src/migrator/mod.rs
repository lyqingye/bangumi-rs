use include_dir::{include_dir, Dir};
use sea_orm_migration::prelude::*;
use std::collections::HashMap;

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../develop/migrations");

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // 获取所有 SQL 文件并排序
        let mut files: Vec<_> = MIGRATIONS_DIR
            .files()
            .filter(|f| f.path().extension().is_some_and(|ext| ext == "sql"))
            .collect();

        files.sort_by_key(|f| f.path().to_string_lossy().to_string());

        // 收集版本化迁移和撤销迁移
        let mut version_migrations = HashMap::new();
        let mut undo_migrations = HashMap::new();

        // 正则表达式匹配版本化迁移和撤销迁移
        // 支持语义化版本号，如: V1.0.0, V1.2.3, V2.0.0 等
        let version_regex = regex::Regex::new(r"V([\d.]+)__([^.]+)\.sql").unwrap();
        let undo_regex = regex::Regex::new(r"U([\d.]+)__([^.]+)\.sql").unwrap();

        // 首先收集所有迁移
        for file in files {
            let file_name = file.path().file_name().unwrap().to_string_lossy();
            let contents = file.contents_utf8().unwrap();

            tracing::info!("loading migration file: {:?}", file.path());

            // 尝试匹配版本化迁移
            if let Some(captures) = version_regex.captures(&file_name) {
                let version = captures.get(1).unwrap().as_str();
                let description = captures.get(2).unwrap().as_str();
                version_migrations.insert(
                    version.to_string(),
                    (description.to_string(), contents.to_string()),
                );
            }
            // 尝试匹配撤销迁移
            else if let Some(captures) = undo_regex.captures(&file_name) {
                let version = captures.get(1).unwrap().as_str();
                let description = captures.get(2).unwrap().as_str();
                undo_migrations.insert(
                    version.to_string(),
                    (description.to_string(), contents.to_string()),
                );
            }
        }

        // 创建迁移实例
        let mut migrations = Vec::new();
        for (version, (description, up_sql)) in version_migrations {
            let down_sql = undo_migrations.get(&version).map(|(_, sql)| sql.clone());

            migrations.push(
                Box::new(Migration::new(&version, &description, &up_sql, down_sql))
                    as Box<dyn MigrationTrait>,
            );
        }

        // 按版本号排序
        migrations.sort_by(|a, b| {
            // 将版本号字符串转换为版本号数组进行比较
            let version_to_parts = |v: &str| -> Vec<u32> {
                v.split('.')
                    .filter_map(|part| part.parse::<u32>().ok())
                    .collect()
            };

            let a_version = version_to_parts(a.name().trim_start_matches('m'));
            let b_version = version_to_parts(b.name().trim_start_matches('m'));
            a_version.cmp(&b_version)
        });

        migrations
    }
}

struct Migration {
    version: String,
    description: String,
    up_sql: String,
    down_sql: Option<String>,
}

impl Migration {
    fn new(version: &str, description: &str, up_sql: &str, down_sql: Option<String>) -> Self {
        Self {
            version: version.to_string(),
            description: description.to_string(),
            up_sql: up_sql.to_string(),
            down_sql,
        }
    }
}

impl MigrationName for Migration {
    fn name(&self) -> &str {
        Box::leak(format!("m{}_{}", self.version, self.description).into_boxed_str())
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(&self.up_sql)
            .await
            .map(|_| ())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if let Some(ref sql) = self.down_sql {
            manager
                .get_connection()
                .execute_unprepared(sql)
                .await
                .map(|_| ())
        } else {
            // 如果没有对应的撤销迁移，直接返回成功
            // 这符合 Flyway 的规范，undo migrations 是可选的
            tracing::warn!(
                "No undo migration available for version {}, description: {}",
                self.version,
                self.description
            );
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_migrations() {
        let mut file_names = vec![
            "V1.0.0__create_anime_table.sql",
            "V3.0.0__create_anime_table.sql",
            "V2.0.0__add_anime_table_columns.sql",
            "V1.0.1__create_anime_table.sql",
        ];

        file_names.sort();

        assert_eq!(
            file_names,
            vec![
                "V1.0.0__create_anime_table.sql",
                "V1.0.1__create_anime_table.sql",
                "V2.0.0__add_anime_table_columns.sql",
                "V3.0.0__create_anime_table.sql",
            ]
        );
    }

    #[test]
    fn test_dir() {
        let migrations = Migrator::migrations();
        for migration in migrations {
            println!("{}", migration.name());
        }
    }
}
