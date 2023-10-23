use std::fmt::Display;

#[derive(Debug)]
pub struct IndexProp {
    columns: Vec<String>,
    to_delete: bool,
}

impl IndexProp {
    pub fn new(columns: &[&str], to_delete: bool) -> Self {
        let mut col = columns
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        col.sort();

        Self {
            to_delete,
            columns: col,
        }
    }
    pub fn name(&self) -> String {
        let mut name = self.columns.join("").to_ascii_lowercase();
        name.truncate(64);
        name
    }

    pub fn concat_columns(&self) -> String {
        self.columns.join(",")
    }

    pub fn delete_index(&self) -> bool {
        self.to_delete
    }
}

#[derive(Debug)]
pub enum IndexType {
    Primary(IndexProp),
    Unique(IndexProp),
    Index(IndexProp),
}

impl Display for IndexType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Primary(index) => {
                if index.delete_index() {
                    write!(f, "DROP PRIMARY KEY {}", index.name())
                } else {
                    write!(
                        f,
                        "ADD PRIMARY KEY {} ({})",
                        index.name(),
                        index.concat_columns()
                    )
                }
            }
            Self::Index(index) => {
                if index.delete_index() {
                    write!(f, "DROP INDEX {}", index.name())
                } else {
                    write!(f, "ADD INDEX {} ({})", index.name(), index.concat_columns())
                }
            }
            Self::Unique(index) => {
                if index.delete_index() {
                    write!(f, "DROP UNIQUE INDEX {}", index.name())
                } else {
                    write!(
                        f,
                        "ADD UNIQUE INDEX {} ({})",
                        index.name(),
                        index.concat_columns()
                    )
                }
            }
        }
    }
}
