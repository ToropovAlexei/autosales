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
                ScalarValue::Int(_) | ScalarValue::Float(_) => {
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
    };
}

#[macro_export]
macro_rules! push_updates {
    ($qb:expr, $( $column:ident => $value:expr ),* $(,)?) => {
        $(
            if let Some(value) = $value {
                $qb.push(concat!(", ", stringify!($column), " = "));
                $qb.push_bind(value);
            }
        )*
    };
}
