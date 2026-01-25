use sqlx::{Postgres, QueryBuilder};

use crate::models::common::{
    AllowedField, FilterValue, ListQuery, Operator, OrderDir, Pagination, ScalarValue,
};

pub fn apply_list_query<'a, F: AllowedField, O: AllowedField>(
    qb: &mut QueryBuilder<'a, Postgres>,
    list: &'a ListQuery<F, O>,
) {
    apply_filters(qb, list);
    apply_order_by(qb, list);
    apply_pagination(qb, &list.pagination);
}

pub fn apply_filters<'a, F: AllowedField, O: AllowedField>(
    qb: &mut QueryBuilder<'a, Postgres>,
    list: &'a ListQuery<F, O>,
) {
    if list.filters.is_empty() {
        return;
    }

    qb.push(" WHERE 1=1");

    for filter in &list.filters {
        let field_col = filter.field.as_ref();
        push_filter_clause(qb, field_col, &filter.op, &filter.value);
    }
}

pub fn apply_order_by<'a, F: AllowedField, O: AllowedField>(
    qb: &mut QueryBuilder<'a, Postgres>,
    list: &'a ListQuery<F, O>,
) {
    if let Some(order_by) = &list.order_by {
        qb.push(" ORDER BY ");
        qb.push(order_by.as_ref());

        match list.order_dir {
            OrderDir::Asc => qb.push(" ASC"),
            OrderDir::Desc => qb.push(" DESC"),
        };
    }
}

fn push_filter_clause<'a>(
    qb: &mut QueryBuilder<'a, Postgres>,
    field_col: &str,
    op: &Operator,
    value: &'a FilterValue,
) {
    match (op, value) {
        (Operator::In, FilterValue::Array(values)) => {
            qb.push(" AND ").push(field_col).push(" IN (");
            let mut sep = qb.separated(", ");
            for v in values {
                match v {
                    ScalarValue::Int(v) => sep.push_bind(v),
                    ScalarValue::Float(v) => sep.push_bind(v),
                    ScalarValue::Bool(v) => sep.push_bind(v),
                    ScalarValue::Text(v) => sep.push_bind(v),
                    ScalarValue::Uuid(v) => sep.push_bind(v),
                    ScalarValue::DateTime(v) => sep.push_bind(v),
                };
            }
            qb.push(")");
        }

        (Operator::Eq | Operator::Ne, FilterValue::Scalar(scalar)) => {
            let op_str = if *op == Operator::Eq { "=" } else { "!=" };
            qb.push(" AND ")
                .push(field_col)
                .push(" ")
                .push(op_str)
                .push(" ");
            push_bind_scalar(qb, scalar);
        }

        (
            op @ (Operator::Gt | Operator::Lt | Operator::Ge | Operator::Le),
            FilterValue::Scalar(scalar),
        ) => {
            let op_str = match op {
                Operator::Gt => ">",
                Operator::Lt => "<",
                Operator::Ge => ">=",
                Operator::Le => "<=",
                _ => return,
            };
            match scalar {
                ScalarValue::Int(_) | ScalarValue::Float(_) | ScalarValue::DateTime(_) => {
                    qb.push(" AND ")
                        .push(field_col)
                        .push(" ")
                        .push(op_str)
                        .push(" ");
                    push_bind_scalar(qb, scalar);
                }
                _ => {}
            }
        }

        (Operator::Like, FilterValue::Scalar(ScalarValue::Text(text))) => {
            qb.push(" AND ").push(field_col).push(" LIKE ");
            qb.push_bind(text.clone());
        }
        (Operator::Contains, FilterValue::Scalar(ScalarValue::Text(text))) => {
            qb.push(" AND ").push(field_col).push(" ILIKE ");
            qb.push_bind(format!("%{}%", text));
        }
        (_, _) => {}
    };
}

pub fn apply_pagination(qb: &mut QueryBuilder<'_, Postgres>, pagination: &Pagination) {
    let limit = pagination.page_size as i64;
    let offset = ((pagination.page - 1) * pagination.page_size) as i64;

    qb.push(" LIMIT ");
    qb.push_bind(limit);

    qb.push(" OFFSET ");
    qb.push_bind(offset);
}

pub fn push_bind_scalar<'a>(qb: &mut QueryBuilder<'a, Postgres>, scalar: &'a ScalarValue) {
    match scalar {
        ScalarValue::Int(v) => qb.push_bind(v),
        ScalarValue::Float(v) => qb.push_bind(v),
        ScalarValue::Bool(v) => qb.push_bind(v),
        ScalarValue::Text(v) => qb.push_bind(v),
        ScalarValue::Uuid(v) => qb.push_bind(v),
        ScalarValue::DateTime(v) => qb.push_bind(v),
    };
}

#[cfg(test)]
mod tests {
    use crate::define_list_query;
    use crate::models::common::Filter;
    use serde::{Deserialize, Serialize};

    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    define_list_query! {
        query_name: TestListQuery,
        filter_fields: {
            TestFilterFields,
            [
                Name => "name",
                Age => "age",
                Active => "active",
                CreatedAt => "created_at",
            ]
        },
        order_fields: {
            TestOrderFields,
            [
                Name => "name",
                Age => "age",
                Active => "active",
                CreatedAt => "created_at",
            ]
        }
    }

    #[test]
    fn test_apply_pagination() {
        let mut qb = QueryBuilder::new("SELECT * FROM test");
        let pagination = Pagination {
            page: 3,
            page_size: 20,
        };
        apply_pagination(&mut qb, &pagination);
        let sql = qb.into_sql();
        assert_eq!(sql, "SELECT * FROM test LIMIT $1 OFFSET $2");
    }

    #[test]
    fn test_apply_order_by() {
        let mut qb = QueryBuilder::new("SELECT * FROM test");
        let query = TestListQuery {
            order_by: Some(TestOrderFields::Name),
            order_dir: OrderDir::Desc,
            ..Default::default()
        };

        apply_order_by(&mut qb, &query);
        let sql = qb.into_sql();
        assert_eq!(sql, "SELECT * FROM test ORDER BY name DESC");
    }

    #[test]
    fn test_apply_filters_empty() {
        let mut qb = QueryBuilder::new("SELECT * FROM test");
        let query = TestListQuery::default();
        apply_filters(&mut qb, &query);
        let sql = qb.into_sql();
        assert_eq!(sql, "SELECT * FROM test");
    }

    #[test]
    fn test_apply_filters_single_eq() {
        let mut qb = QueryBuilder::new("SELECT * FROM test");
        let mut query = TestListQuery::default();
        query.filters.push(Filter {
            field: TestFilterFields::Name,
            op: Operator::Eq,
            value: FilterValue::Scalar(ScalarValue::Text("test".to_string())),
        });

        apply_filters(&mut qb, &query);
        let sql = qb.into_sql();
        assert_eq!(sql, "SELECT * FROM test WHERE 1=1 AND name = $1");
    }

    #[test]
    fn test_apply_filters_multiple() {
        let mut qb = QueryBuilder::new("SELECT * FROM test");
        let mut query = TestListQuery::default();
        query.filters.push(Filter {
            field: TestFilterFields::Name,
            op: Operator::Like,
            value: FilterValue::Scalar(ScalarValue::Text("test%".to_string())),
        });
        query.filters.push(Filter {
            field: TestFilterFields::Age,
            op: Operator::Ge,
            value: FilterValue::Scalar(ScalarValue::Int(18)),
        });

        apply_filters(&mut qb, &query);
        let sql = qb.into_sql();
        assert_eq!(
            sql,
            "SELECT * FROM test WHERE 1=1 AND name LIKE $1 AND age >= $2"
        );
    }

    #[test]
    fn test_filter_in_clause() {
        let mut qb = QueryBuilder::new("SELECT * FROM test");
        let mut query = TestListQuery::default();
        query.filters.push(Filter {
            field: TestFilterFields::Age,
            op: Operator::In,
            value: FilterValue::Array(vec![
                ScalarValue::Int(18),
                ScalarValue::Int(21),
                ScalarValue::Int(30),
            ]),
        });

        apply_filters(&mut qb, &query);
        let sql = qb.into_sql();
        assert_eq!(sql, "SELECT * FROM test WHERE 1=1 AND age IN ($1, $2, $3)");
    }

    #[test]
    fn test_filter_datetime_clause() {
        let mut qb = QueryBuilder::new("SELECT * FROM test");
        let mut query = TestListQuery::default();
        let now = Utc::now();
        query.filters.push(Filter {
            field: TestFilterFields::CreatedAt,
            op: Operator::Le,
            value: FilterValue::Scalar(ScalarValue::DateTime(now)),
        });

        apply_filters(&mut qb, &query);
        let sql = qb.into_sql();
        assert_eq!(sql, "SELECT * FROM test WHERE 1=1 AND created_at <= $1");
    }

    #[test]
    fn test_filter_unsupported_op_for_type_is_ignored() {
        let mut qb = QueryBuilder::new("SELECT * FROM test");
        let mut query = TestListQuery::default();
        // Booleans don't support Greater Than
        query.filters.push(Filter {
            field: TestFilterFields::Active,
            op: Operator::Gt,
            value: FilterValue::Scalar(ScalarValue::Bool(true)),
        });

        apply_filters(&mut qb, &query);
        let sql = qb.into_sql();
        // Should not add any WHERE clause for the unsupported filter
        assert_eq!(sql, "SELECT * FROM test WHERE 1=1");
    }

    #[test]
    fn test_apply_list_query_full() {
        let mut qb = QueryBuilder::new("SELECT * FROM test");
        let mut query = TestListQuery::default();

        // Filters
        query.filters.push(Filter {
            field: TestFilterFields::Active,
            op: Operator::Eq,
            value: FilterValue::Scalar(ScalarValue::Bool(true)),
        });
        query.filters.push(Filter {
            field: TestFilterFields::Name,
            op: Operator::Contains,
            value: FilterValue::Scalar(ScalarValue::Text("admin".to_string())),
        });

        // Order
        query.order_by = Some(TestOrderFields::CreatedAt);
        query.order_dir = OrderDir::Asc;

        // Pagination
        query.pagination = Pagination {
            page: 1,
            page_size: 10,
        };

        apply_list_query(&mut qb, &query);
        let sql = qb.into_sql();
        assert_eq!(
            sql,
            "SELECT * FROM test WHERE 1=1 AND active = $1 AND name ILIKE $2 ORDER BY created_at ASC LIMIT $3 OFFSET $4"
        );
    }

    #[test]
    fn test_push_bind_scalar_all_types() {
        let text = ScalarValue::Text("text".to_string());
        let uuid = ScalarValue::Uuid(Uuid::nil());
        let dt = ScalarValue::DateTime(Utc::now());
        let mut qb = QueryBuilder::new("");
        push_bind_scalar(&mut qb, &ScalarValue::Int(1));
        push_bind_scalar(&mut qb, &ScalarValue::Float(1.5));
        push_bind_scalar(&mut qb, &ScalarValue::Bool(true));
        push_bind_scalar(&mut qb, &text);
        push_bind_scalar(&mut qb, &uuid);
        push_bind_scalar(&mut qb, &dt);

        // We can't easily check the bound values here, but we can ensure
        // the SQL string has the correct number of bind parameters.
        let sql = qb.into_sql();
        assert_eq!(sql, "$1$2$3$4$5$6");
    }
}
