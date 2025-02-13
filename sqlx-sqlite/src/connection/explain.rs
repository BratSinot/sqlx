use crate::connection::{execute, ConnectionState};
use crate::error::Error;
use crate::from_row::FromRow;
use crate::type_info::DataType;
use crate::SqliteTypeInfo;
use sqlx_core::HashMap;
use std::collections::HashSet;
use std::str::from_utf8;

// affinity
const SQLITE_AFF_NONE: u8 = 0x40; /* '@' */
const SQLITE_AFF_BLOB: u8 = 0x41; /* 'A' */
const SQLITE_AFF_TEXT: u8 = 0x42; /* 'B' */
const SQLITE_AFF_NUMERIC: u8 = 0x43; /* 'C' */
const SQLITE_AFF_INTEGER: u8 = 0x44; /* 'D' */
const SQLITE_AFF_REAL: u8 = 0x45; /* 'E' */

// opcodes
const OP_INIT: &str = "Init";
const OP_GOTO: &str = "Goto";
const OP_DECR_JUMP_ZERO: &str = "DecrJumpZero";
const OP_DELETE: &str = "Delete";
const OP_ELSE_EQ: &str = "ElseEq";
const OP_EQ: &str = "Eq";
const OP_END_COROUTINE: &str = "EndCoroutine";
const OP_FILTER: &str = "Filter";
const OP_FK_IF_ZERO: &str = "FkIfZero";
const OP_FOUND: &str = "Found";
const OP_GE: &str = "Ge";
const OP_GO_SUB: &str = "Gosub";
const OP_GT: &str = "Gt";
const OP_IDX_GE: &str = "IdxGE";
const OP_IDX_GT: &str = "IdxGT";
const OP_IDX_LE: &str = "IdxLE";
const OP_IDX_LT: &str = "IdxLT";
const OP_IF: &str = "If";
const OP_IF_NO_HOPE: &str = "IfNoHope";
const OP_IF_NOT: &str = "IfNot";
const OP_IF_NOT_OPEN: &str = "IfNotOpen";
const OP_IF_NOT_ZERO: &str = "IfNotZero";
const OP_IF_NULL_ROW: &str = "IfNullRow";
const OP_IF_POS: &str = "IfPos";
const OP_IF_SMALLER: &str = "IfSmaller";
const OP_INCR_VACUUM: &str = "IncrVacuum";
const OP_INIT_COROUTINE: &str = "InitCoroutine";
const OP_IS_NULL: &str = "IsNull";
const OP_IS_NULL_OR_TYPE: &str = "IsNullOrType";
const OP_LAST: &str = "Last";
const OP_LE: &str = "Le";
const OP_LT: &str = "Lt";
const OP_MUST_BE_INT: &str = "MustBeInt";
const OP_NE: &str = "Ne";
const OP_NEXT: &str = "Next";
const OP_NO_CONFLICT: &str = "NoConflict";
const OP_NOT_EXISTS: &str = "NotExists";
const OP_NOT_NULL: &str = "NotNull";
const OP_ONCE: &str = "Once";
const OP_PREV: &str = "Prev";
const OP_PROGRAM: &str = "Program";
const OP_RETURN: &str = "Return";
const OP_REWIND: &str = "Rewind";
const OP_ROW_DATA: &str = "RowData";
const OP_ROW_SET_READ: &str = "RowSetRead";
const OP_ROW_SET_TEST: &str = "RowSetTest";
const OP_SEEK_GE: &str = "SeekGE";
const OP_SEEK_GT: &str = "SeekGT";
const OP_SEEK_LE: &str = "SeekLE";
const OP_SEEK_LT: &str = "SeekLT";
const OP_SEEK_ROW_ID: &str = "SeekRowId";
const OP_SEEK_SCAN: &str = "SeekScan";
const OP_SEQUENCE: &str = "Sequence";
const OP_SEQUENCE_TEST: &str = "SequenceTest";
const OP_SORT: &str = "Sort";
const OP_SORTER_DATA: &str = "SorterData";
const OP_SORTER_INSERT: &str = "SorterInsert";
const OP_SORTER_NEXT: &str = "SorterNext";
const OP_SORTER_OPEN: &str = "SorterOpen";
const OP_SORTER_SORT: &str = "SorterSort";
const OP_V_FILTER: &str = "VFilter";
const OP_V_NEXT: &str = "VNext";
const OP_YIELD: &str = "Yield";
const OP_JUMP: &str = "Jump";
const OP_COLUMN: &str = "Column";
const OP_MAKE_RECORD: &str = "MakeRecord";
const OP_INSERT: &str = "Insert";
const OP_IDX_INSERT: &str = "IdxInsert";
const OP_OPEN_PSEUDO: &str = "OpenPseudo";
const OP_OPEN_READ: &str = "OpenRead";
const OP_OPEN_WRITE: &str = "OpenWrite";
const OP_OPEN_EPHEMERAL: &str = "OpenEphemeral";
const OP_OPEN_AUTOINDEX: &str = "OpenAutoindex";
const OP_AGG_FINAL: &str = "AggFinal";
const OP_AGG_VALUE: &str = "AggValue";
const OP_AGG_STEP: &str = "AggStep";
const OP_FUNCTION: &str = "Function";
const OP_MOVE: &str = "Move";
const OP_COPY: &str = "Copy";
const OP_SCOPY: &str = "SCopy";
const OP_NULL: &str = "Null";
const OP_NULL_ROW: &str = "NullRow";
const OP_INT_COPY: &str = "IntCopy";
const OP_CAST: &str = "Cast";
const OP_STRING8: &str = "String8";
const OP_INT64: &str = "Int64";
const OP_INTEGER: &str = "Integer";
const OP_REAL: &str = "Real";
const OP_NOT: &str = "Not";
const OP_BLOB: &str = "Blob";
const OP_VARIABLE: &str = "Variable";
const OP_COUNT: &str = "Count";
const OP_ROWID: &str = "Rowid";
const OP_NEWROWID: &str = "NewRowid";
const OP_OR: &str = "Or";
const OP_AND: &str = "And";
const OP_BIT_AND: &str = "BitAnd";
const OP_BIT_OR: &str = "BitOr";
const OP_SHIFT_LEFT: &str = "ShiftLeft";
const OP_SHIFT_RIGHT: &str = "ShiftRight";
const OP_ADD: &str = "Add";
const OP_SUBTRACT: &str = "Subtract";
const OP_MULTIPLY: &str = "Multiply";
const OP_DIVIDE: &str = "Divide";
const OP_REMAINDER: &str = "Remainder";
const OP_CONCAT: &str = "Concat";
const OP_OFFSET_LIMIT: &str = "OffsetLimit";
const OP_RESULT_ROW: &str = "ResultRow";
const OP_HALT: &str = "Halt";

const MAX_LOOP_COUNT: u8 = 2;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum ColumnType {
    Single {
        datatype: DataType,
        nullable: Option<bool>,
    },
    Record(Vec<ColumnType>),
}

impl Default for ColumnType {
    fn default() -> Self {
        Self::Single {
            datatype: DataType::Null,
            nullable: None,
        }
    }
}

impl ColumnType {
    fn null() -> Self {
        Self::Single {
            datatype: DataType::Null,
            nullable: Some(true),
        }
    }
    fn map_to_datatype(&self) -> DataType {
        match self {
            Self::Single { datatype, .. } => datatype.clone(),
            Self::Record(_) => DataType::Null, //If we're trying to coerce to a regular Datatype, we can assume a Record is invalid for the context
        }
    }
    fn map_to_nullable(&self) -> Option<bool> {
        match self {
            Self::Single { nullable, .. } => *nullable,
            Self::Record(_) => None, //If we're trying to coerce to a regular Datatype, we can assume a Record is invalid for the context
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum RegDataType {
    Single(ColumnType),
    Int(i64),
}

impl RegDataType {
    fn map_to_datatype(&self) -> DataType {
        match self {
            RegDataType::Single(d) => d.map_to_datatype(),
            RegDataType::Int(_) => DataType::Int,
        }
    }
    fn map_to_nullable(&self) -> Option<bool> {
        match self {
            RegDataType::Single(d) => d.map_to_nullable(),
            RegDataType::Int(_) => Some(false),
        }
    }
    fn map_to_columntype(&self) -> ColumnType {
        match self {
            RegDataType::Single(d) => d.clone(),
            RegDataType::Int(_) => ColumnType::Single {
                datatype: DataType::Int,
                nullable: Some(false),
            },
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum CursorDataType {
    Normal {
        cols: HashMap<i64, ColumnType>,
        is_empty: Option<bool>,
    },
    Pseudo(i64),
}

impl CursorDataType {
    fn from_sparse_record(record: &HashMap<i64, ColumnType>, is_empty: Option<bool>) -> Self {
        Self::Normal {
            cols: record
                .iter()
                .map(|(colnum, datatype)| (*colnum, datatype.clone()))
                .collect(),
            is_empty,
        }
    }

    fn from_dense_record(record: &Vec<ColumnType>, is_empty: Option<bool>) -> Self {
        Self::Normal {
            cols: (0..).zip(record.iter().cloned()).collect(),
            is_empty,
        }
    }

    fn map_to_dense_record(&self, registers: &HashMap<i64, RegDataType>) -> Vec<ColumnType> {
        match self {
            Self::Normal { cols, .. } => {
                let mut rowdata = vec![ColumnType::default(); cols.len()];
                for (idx, col) in cols.iter() {
                    rowdata[*idx as usize] = col.clone();
                }
                rowdata
            }
            Self::Pseudo(i) => match registers.get(i) {
                Some(RegDataType::Single(ColumnType::Record(r))) => r.clone(),
                _ => Vec::new(),
            },
        }
    }

    fn map_to_sparse_record(
        &self,
        registers: &HashMap<i64, RegDataType>,
    ) -> HashMap<i64, ColumnType> {
        match self {
            Self::Normal { cols, .. } => cols.clone(),
            Self::Pseudo(i) => match registers.get(i) {
                Some(RegDataType::Single(ColumnType::Record(r))) => {
                    (0..).zip(r.iter().cloned()).collect()
                }
                _ => HashMap::new(),
            },
        }
    }

    fn is_empty(&self) -> Option<bool> {
        match self {
            Self::Normal { is_empty, .. } => *is_empty,
            Self::Pseudo(_) => Some(false), //pseudo cursors have exactly one row
        }
    }
}

#[allow(clippy::wildcard_in_or_patterns)]
fn affinity_to_type(affinity: u8) -> DataType {
    match affinity {
        SQLITE_AFF_BLOB => DataType::Blob,
        SQLITE_AFF_INTEGER => DataType::Int64,
        SQLITE_AFF_NUMERIC => DataType::Numeric,
        SQLITE_AFF_REAL => DataType::Float,
        SQLITE_AFF_TEXT => DataType::Text,

        SQLITE_AFF_NONE | _ => DataType::Null,
    }
}

#[allow(clippy::wildcard_in_or_patterns)]
fn opcode_to_type(op: &str) -> DataType {
    match op {
        OP_REAL => DataType::Float,
        OP_BLOB => DataType::Blob,
        OP_AND | OP_OR => DataType::Bool,
        OP_ROWID | OP_COUNT | OP_INT64 | OP_INTEGER => DataType::Int64,
        OP_STRING8 => DataType::Text,
        OP_COLUMN | _ => DataType::Null,
    }
}

fn root_block_columns(
    conn: &mut ConnectionState,
) -> Result<HashMap<(i64, i64), HashMap<i64, ColumnType>>, Error> {
    let table_block_columns: Vec<(i64, i64, i64, String, bool)> = execute::iter(
        conn,
        "SELECT s.dbnum, s.rootpage, col.cid as colnum, col.type, col.\"notnull\"
         FROM (
             select 1 dbnum, tss.* from temp.sqlite_schema tss
             UNION ALL select 0 dbnum, mss.* from main.sqlite_schema mss
             ) s
         JOIN pragma_table_info(s.name) AS col
         WHERE s.type = 'table'
         UNION ALL
         SELECT s.dbnum, s.rootpage, idx.seqno as colnum, col.type, col.\"notnull\"
         FROM (
             select 1 dbnum, tss.* from temp.sqlite_schema tss
             UNION ALL select 0 dbnum, mss.* from main.sqlite_schema mss
             ) s
         JOIN pragma_index_info(s.name) AS idx
         LEFT JOIN pragma_table_info(s.tbl_name) as col
           ON col.cid = idx.cid
           WHERE s.type = 'index'",
        None,
        false,
    )?
    .filter_map(|res| res.map(|either| either.right()).transpose())
    .map(|row| FromRow::from_row(&row?))
    .collect::<Result<Vec<_>, Error>>()?;

    let mut row_info: HashMap<(i64, i64), HashMap<i64, ColumnType>> = HashMap::new();
    for (dbnum, block, colnum, datatype, notnull) in table_block_columns {
        let row_info = row_info.entry((dbnum, block)).or_default();
        row_info.insert(
            colnum,
            ColumnType::Single {
                datatype: datatype.parse().unwrap_or(DataType::Null),
                nullable: Some(!notnull),
            },
        );
    }

    return Ok(row_info);
}

#[derive(Debug, Clone, PartialEq)]
struct QueryState {
    // The number of times each instruction has been visited
    pub visited: Vec<u8>,
    // A log of the order of execution of each instruction
    pub history: Vec<usize>,
    // Registers
    pub r: HashMap<i64, RegDataType>,
    // Rows that pointers point to
    pub p: HashMap<i64, CursorDataType>,
    // Next instruction to execute
    pub program_i: usize,
    // Results published by the execution
    pub result: Option<Vec<(Option<SqliteTypeInfo>, Option<bool>)>>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct BranchStateHash {
    instruction: usize,
    //register index, data type
    registers: Vec<(i64, RegDataType)>,
    //cursor index, is_empty, pseudo register index
    cursor_metadata: Vec<(i64, Option<bool>, Option<i64>)>,
    //cursor index, column index, data type
    cursors: Vec<(i64, i64, Option<ColumnType>)>,
}

impl BranchStateHash {
    pub fn from_query_state(st: &QueryState) -> Self {
        let mut reg = vec![];
        for (k, v) in &st.r {
            reg.push((*k, v.clone()));
        }
        reg.sort_by_key(|v| v.0);

        let mut cur = vec![];
        let mut cur_meta = vec![];
        for (k, v) in &st.p {
            match v {
                CursorDataType::Normal { cols, is_empty } => {
                    cur_meta.push((*k, *is_empty, None));
                    for (i, col) in cols {
                        cur.push((*k, *i, Some(col.clone())));
                    }
                }
                CursorDataType::Pseudo(i) => {
                    cur_meta.push((*k, None, Some(*i)));
                    //don't bother copying columns, they are in register i
                }
            }
        }
        cur_meta.sort_by(|a, b| a.0.cmp(&b.0));
        cur.sort_by(|a, b| {
            if a.0 == b.0 {
                a.1.cmp(&b.1)
            } else {
                a.0.cmp(&b.0)
            }
        });
        Self {
            instruction: st.program_i,
            registers: reg,
            cursor_metadata: cur_meta,
            cursors: cur,
        }
    }
}

// Opcode Reference: https://sqlite.org/opcode.html
pub(super) fn explain(
    conn: &mut ConnectionState,
    query: &str,
) -> Result<(Vec<SqliteTypeInfo>, Vec<Option<bool>>), Error> {
    let root_block_cols = root_block_columns(conn)?;
    let program: Vec<(i64, String, i64, i64, i64, Vec<u8>)> =
        execute::iter(conn, &format!("EXPLAIN {}", query), None, false)?
            .filter_map(|res| res.map(|either| either.right()).transpose())
            .map(|row| FromRow::from_row(&row?))
            .collect::<Result<Vec<_>, Error>>()?;
    let program_size = program.len();

    let mut logger =
        crate::logger::QueryPlanLogger::new(query, &program, conn.log_settings.clone());

    let mut states = vec![QueryState {
        visited: vec![0; program_size],
        history: Vec::new(),
        r: HashMap::with_capacity(6),
        p: HashMap::with_capacity(6),
        program_i: 0,
        result: None,
    }];

    let mut visited_branch_state: HashSet<BranchStateHash> = HashSet::new();

    let mut result_states = Vec::new();

    while let Some(mut state) = states.pop() {
        while state.program_i < program_size {
            let (_, ref opcode, p1, p2, p3, ref p4) = program[state.program_i];
            state.history.push(state.program_i);

            if state.visited[state.program_i] > MAX_LOOP_COUNT {
                if logger.log_enabled() {
                    let program_history: Vec<&(i64, String, i64, i64, i64, Vec<u8>)> =
                        state.history.iter().map(|i| &program[*i]).collect();
                    logger.add_result((program_history, None));
                }

                //avoid (infinite) loops by breaking if we ever hit the same instruction twice
                break;
            }

            state.visited[state.program_i] += 1;

            match &**opcode {
                OP_INIT => {
                    // start at <p2>
                    state.program_i = p2 as usize;
                    continue;
                }

                OP_GOTO => {
                    // goto <p2>

                    state.program_i = p2 as usize;
                    continue;
                }

                OP_GO_SUB => {
                    // store current instruction in r[p1], goto <p2>
                    state.r.insert(p1, RegDataType::Int(state.program_i as i64));
                    state.program_i = p2 as usize;
                    continue;
                }

                OP_DECR_JUMP_ZERO | OP_ELSE_EQ | OP_EQ | OP_FILTER | OP_FK_IF_ZERO | OP_FOUND
                | OP_GE | OP_GT | OP_IDX_GE | OP_IDX_GT | OP_IDX_LE | OP_IDX_LT | OP_IF_NO_HOPE
                | OP_IF_NOT | OP_IF_NOT_OPEN | OP_IF_NOT_ZERO | OP_IF_NULL_ROW | OP_IF_SMALLER
                | OP_INCR_VACUUM | OP_IS_NULL | OP_IS_NULL_OR_TYPE | OP_LE | OP_LT | OP_NE
                | OP_NEXT | OP_NO_CONFLICT | OP_NOT_EXISTS | OP_ONCE | OP_PREV | OP_PROGRAM
                | OP_ROW_SET_READ | OP_ROW_SET_TEST | OP_SEEK_GE | OP_SEEK_GT | OP_SEEK_LE
                | OP_SEEK_LT | OP_SEEK_ROW_ID | OP_SEEK_SCAN | OP_SEQUENCE_TEST
                | OP_SORTER_NEXT | OP_V_FILTER | OP_V_NEXT => {
                    // goto <p2> or next instruction (depending on actual values)

                    let mut branch_state = state.clone();
                    branch_state.program_i = p2 as usize;

                    let bs_hash = BranchStateHash::from_query_state(&branch_state);
                    if !visited_branch_state.contains(&bs_hash) {
                        visited_branch_state.insert(bs_hash);
                        states.push(branch_state);
                    }

                    state.program_i += 1;
                    continue;
                }

                OP_NOT_NULL => {
                    // goto <p2> or next instruction (depending on actual values)

                    let might_branch = match state.r.get(&p1) {
                        Some(r_p1) => !matches!(r_p1.map_to_datatype(), DataType::Null),
                        _ => false,
                    };

                    let might_not_branch = match state.r.get(&p1) {
                        Some(r_p1) => !matches!(r_p1.map_to_nullable(), Some(false)),
                        _ => false,
                    };

                    if might_branch {
                        let mut branch_state = state.clone();
                        branch_state.program_i = p2 as usize;
                        if let Some(RegDataType::Single(ColumnType::Single { nullable, .. })) =
                            branch_state.r.get_mut(&p1)
                        {
                            *nullable = Some(false);
                        }

                        let bs_hash = BranchStateHash::from_query_state(&branch_state);
                        if !visited_branch_state.contains(&bs_hash) {
                            visited_branch_state.insert(bs_hash);
                            states.push(branch_state);
                        }
                    }

                    if might_not_branch {
                        state.program_i += 1;
                        state
                            .r
                            .insert(p1, RegDataType::Single(ColumnType::default()));
                        continue;
                    } else {
                        break;
                    }
                }

                OP_MUST_BE_INT => {
                    // if p1 can be coerced to int, continue
                    // if p1 cannot be coerced to int, error if p2 == 0, else jump to p2

                    //don't bother checking actual types, just don't branch to instruction 0
                    if p2 != 0 {
                        let mut branch_state = state.clone();
                        branch_state.program_i = p2 as usize;

                        let bs_hash = BranchStateHash::from_query_state(&branch_state);
                        if !visited_branch_state.contains(&bs_hash) {
                            visited_branch_state.insert(bs_hash);
                            states.push(branch_state);
                        }
                    }

                    state.program_i += 1;
                    continue;
                }

                OP_IF => {
                    // goto <p2> if r[p1] is true (1) or r[p1] is null and p3 is nonzero

                    let might_branch = match state.r.get(&p1) {
                        Some(RegDataType::Int(r_p1)) => *r_p1 != 0,
                        _ => true,
                    };

                    let might_not_branch = match state.r.get(&p1) {
                        Some(RegDataType::Int(r_p1)) => *r_p1 == 0,
                        _ => true,
                    };

                    if might_branch {
                        let mut branch_state = state.clone();
                        branch_state.program_i = p2 as usize;
                        if p3 == 0 {
                            branch_state.r.insert(p1, RegDataType::Int(1));
                        }

                        let bs_hash = BranchStateHash::from_query_state(&branch_state);
                        if !visited_branch_state.contains(&bs_hash) {
                            visited_branch_state.insert(bs_hash);
                            states.push(branch_state);
                        }
                    }

                    if might_not_branch {
                        state.program_i += 1;
                        if p3 == 0 {
                            state.r.insert(p1, RegDataType::Int(0));
                        }
                        continue;
                    } else {
                        break;
                    }
                }

                OP_IF_POS => {
                    // goto <p2> if r[p1] is true (1) or r[p1] is null and p3 is nonzero

                    // as a workaround for large offset clauses, both branches will be attempted after 1 loop

                    let might_branch = match state.r.get(&p1) {
                        Some(RegDataType::Int(r_p1)) => *r_p1 >= 1,
                        _ => true,
                    };

                    let might_not_branch = match state.r.get(&p1) {
                        Some(RegDataType::Int(r_p1)) => *r_p1 < 1,
                        _ => true,
                    };

                    let loop_detected = state.visited[state.program_i] > 1;
                    if might_branch || loop_detected {
                        let mut branch_state = state.clone();
                        branch_state.program_i = p2 as usize;
                        if let Some(RegDataType::Int(r_p1)) = branch_state.r.get_mut(&p1) {
                            *r_p1 -= 1;
                        }
                        states.push(branch_state);
                    }

                    if might_not_branch {
                        state.program_i += 1;
                        continue;
                    } else if loop_detected {
                        state.program_i += 1;
                        if matches!(state.r.get_mut(&p1), Some(RegDataType::Int(..))) {
                            //forget the exact value, in case some later cares
                            state.r.insert(
                                p1,
                                RegDataType::Single(ColumnType::Single {
                                    datatype: DataType::Int64,
                                    nullable: Some(false),
                                }),
                            );
                        }
                        continue;
                    } else {
                        break;
                    }
                }

                OP_REWIND | OP_LAST | OP_SORT | OP_SORTER_SORT => {
                    // goto <p2> if cursor p1 is empty and p2 != 0, else next instruction

                    if p2 == 0 {
                        state.program_i += 1;
                        continue;
                    }

                    if let Some(cursor) = state.p.get(&p1) {
                        if matches!(cursor.is_empty(), None | Some(true)) {
                            //only take this branch if the cursor is empty

                            let mut branch_state = state.clone();
                            branch_state.program_i = p2 as usize;

                            if let Some(CursorDataType::Normal { is_empty, .. }) =
                                branch_state.p.get_mut(&p1)
                            {
                                *is_empty = Some(true);
                            }
                            states.push(branch_state);
                        }

                        if matches!(cursor.is_empty(), None | Some(false)) {
                            //only take this branch if the cursor is non-empty
                            state.program_i += 1;
                            continue;
                        } else {
                            break;
                        }
                    }

                    if logger.log_enabled() {
                        let program_history: Vec<&(i64, String, i64, i64, i64, Vec<u8>)> =
                            state.history.iter().map(|i| &program[*i]).collect();
                        logger.add_result((program_history, None));
                    }

                    break;
                }

                OP_INIT_COROUTINE => {
                    // goto <p2> or next instruction (depending on actual values)

                    state.r.insert(p1, RegDataType::Int(p3));

                    if p2 != 0 {
                        state.program_i = p2 as usize;
                    } else {
                        state.program_i += 1;
                    }
                    continue;
                }

                OP_END_COROUTINE => {
                    // jump to p2 of the yield instruction pointed at by register p1

                    if let Some(RegDataType::Int(yield_i)) = state.r.get(&p1) {
                        if let Some((_, yield_op, _, yield_p2, _, _)) =
                            program.get(*yield_i as usize)
                        {
                            if OP_YIELD == yield_op.as_str() {
                                state.program_i = (*yield_p2) as usize;
                                state.r.remove(&p1);
                                continue;
                            } else {
                                if logger.log_enabled() {
                                    let program_history: Vec<&(
                                        i64,
                                        String,
                                        i64,
                                        i64,
                                        i64,
                                        Vec<u8>,
                                    )> = state.history.iter().map(|i| &program[*i]).collect();
                                    logger.add_result((program_history, None));
                                }

                                break;
                            }
                        } else {
                            if logger.log_enabled() {
                                let program_history: Vec<&(i64, String, i64, i64, i64, Vec<u8>)> =
                                    state.history.iter().map(|i| &program[*i]).collect();
                                logger.add_result((program_history, None));
                            }
                            break;
                        }
                    } else {
                        if logger.log_enabled() {
                            let program_history: Vec<&(i64, String, i64, i64, i64, Vec<u8>)> =
                                state.history.iter().map(|i| &program[*i]).collect();
                            logger.add_result((program_history, None));
                        }
                        break;
                    }
                }

                OP_RETURN => {
                    // jump to the instruction after the instruction pointed at by register p1

                    if let Some(RegDataType::Int(return_i)) = state.r.get(&p1) {
                        state.program_i = (*return_i + 1) as usize;
                        state.r.remove(&p1);
                        continue;
                    } else {
                        if logger.log_enabled() {
                            let program_history: Vec<&(i64, String, i64, i64, i64, Vec<u8>)> =
                                state.history.iter().map(|i| &program[*i]).collect();
                            logger.add_result((program_history, None));
                        }
                        break;
                    }
                }

                OP_YIELD => {
                    // jump to p2 of the yield instruction pointed at by register p1, store prior instruction in p1

                    if let Some(RegDataType::Int(yield_i)) = state.r.get_mut(&p1) {
                        let program_i: usize = state.program_i;

                        //if yielding to a yield operation, go to the NEXT instruction after that instruction
                        if program
                            .get(*yield_i as usize)
                            .map(|(_, yield_op, _, _, _, _)| yield_op.as_str())
                            == Some(OP_YIELD)
                        {
                            state.program_i = (*yield_i + 1) as usize;
                            *yield_i = program_i as i64;
                            continue;
                        } else {
                            state.program_i = *yield_i as usize;
                            *yield_i = program_i as i64;
                            continue;
                        }
                    } else {
                        if logger.log_enabled() {
                            let program_history: Vec<&(i64, String, i64, i64, i64, Vec<u8>)> =
                                state.history.iter().map(|i| &program[*i]).collect();
                            logger.add_result((program_history, None));
                        }
                        break;
                    }
                }

                OP_JUMP => {
                    // goto one of <p1>, <p2>, or <p3> based on the result of a prior compare

                    let mut branch_state = state.clone();
                    branch_state.program_i = p1 as usize;
                    let bs_hash = BranchStateHash::from_query_state(&branch_state);
                    if !visited_branch_state.contains(&bs_hash) {
                        visited_branch_state.insert(bs_hash);
                        states.push(branch_state);
                    }

                    let mut branch_state = state.clone();
                    branch_state.program_i = p2 as usize;
                    let bs_hash = BranchStateHash::from_query_state(&branch_state);
                    if !visited_branch_state.contains(&bs_hash) {
                        visited_branch_state.insert(bs_hash);
                        states.push(branch_state);
                    }

                    let mut branch_state = state.clone();
                    branch_state.program_i = p3 as usize;
                    let bs_hash = BranchStateHash::from_query_state(&branch_state);
                    if !visited_branch_state.contains(&bs_hash) {
                        visited_branch_state.insert(bs_hash);
                        states.push(branch_state);
                    }
                }

                OP_COLUMN => {
                    //Get the row stored at p1, or NULL; get the column stored at p2, or NULL
                    if let Some(record) = state.p.get(&p1).map(|c| c.map_to_sparse_record(&state.r))
                    {
                        if let Some(col) = record.get(&p2) {
                            // insert into p3 the datatype of the col
                            state.r.insert(p3, RegDataType::Single(col.clone()));
                        } else {
                            state
                                .r
                                .insert(p3, RegDataType::Single(ColumnType::default()));
                        }
                    } else {
                        state
                            .r
                            .insert(p3, RegDataType::Single(ColumnType::default()));
                    }
                }

                OP_SEQUENCE => {
                    //Copy sequence number from cursor p1 to register p2, increment cursor p1 sequence number

                    //Cursor emulation doesn't sequence value, but it is an int
                    state.r.insert(
                        p2,
                        RegDataType::Single(ColumnType::Single {
                            datatype: DataType::Int64,
                            nullable: Some(false),
                        }),
                    );
                }

                OP_ROW_DATA | OP_SORTER_DATA => {
                    //Get entire row from cursor p1, store it into register p2
                    if let Some(record) = state.p.get(&p1) {
                        let rowdata = record.map_to_dense_record(&state.r);
                        state
                            .r
                            .insert(p2, RegDataType::Single(ColumnType::Record(rowdata)));
                    } else {
                        state
                            .r
                            .insert(p2, RegDataType::Single(ColumnType::Record(Vec::new())));
                    }
                }

                OP_MAKE_RECORD => {
                    // p3 = Record([p1 .. p1 + p2])
                    let mut record = Vec::with_capacity(p2 as usize);
                    for reg in p1..p1 + p2 {
                        record.push(
                            state
                                .r
                                .get(&reg)
                                .map(|d| d.clone().map_to_columntype())
                                .unwrap_or(ColumnType::default()),
                        );
                    }
                    state
                        .r
                        .insert(p3, RegDataType::Single(ColumnType::Record(record)));
                }

                OP_INSERT | OP_IDX_INSERT | OP_SORTER_INSERT => {
                    if let Some(RegDataType::Single(ColumnType::Record(record))) = state.r.get(&p2)
                    {
                        if let Some(CursorDataType::Normal { cols, is_empty }) =
                            state.p.get_mut(&p1)
                        {
                            // Insert the record into wherever pointer p1 is
                            *cols = (0..).zip(record.iter().cloned()).collect();
                            *is_empty = Some(false);
                        }
                    }
                    //Noop if the register p2 isn't a record, or if pointer p1 does not exist
                }

                OP_DELETE => {
                    // delete a record from cursor p1
                    if let Some(CursorDataType::Normal { is_empty, .. }) = state.p.get_mut(&p1) {
                        if *is_empty == Some(false) {
                            *is_empty = None; //the cursor might be empty now
                        }
                    }
                }

                OP_OPEN_PSEUDO => {
                    // Create a cursor p1 aliasing the record from register p2
                    state.p.insert(p1, CursorDataType::Pseudo(p2));
                }

                OP_OPEN_READ | OP_OPEN_WRITE => {
                    //Create a new pointer which is referenced by p1, take column metadata from db schema if found
                    if p3 == 0 || p3 == 1 {
                        if let Some(columns) = root_block_cols.get(&(p3, p2)) {
                            state
                                .p
                                .insert(p1, CursorDataType::from_sparse_record(columns, None));
                        } else {
                            state.p.insert(
                                p1,
                                CursorDataType::Normal {
                                    cols: HashMap::with_capacity(6),
                                    is_empty: None,
                                },
                            );
                        }
                    } else {
                        state.p.insert(
                            p1,
                            CursorDataType::Normal {
                                cols: HashMap::with_capacity(6),
                                is_empty: None,
                            },
                        );
                    }
                }

                OP_OPEN_EPHEMERAL | OP_OPEN_AUTOINDEX | OP_SORTER_OPEN => {
                    //Create a new pointer which is referenced by p1
                    state.p.insert(
                        p1,
                        CursorDataType::from_dense_record(
                            &vec![ColumnType::null(); p2 as usize],
                            Some(true),
                        ),
                    );
                }

                OP_VARIABLE => {
                    // r[p2] = <value of variable>
                    state.r.insert(p2, RegDataType::Single(ColumnType::null()));
                }

                OP_FUNCTION => {
                    // r[p1] = func( _ )
                    match from_utf8(p4).map_err(Error::protocol)? {
                        "last_insert_rowid(0)" => {
                            // last_insert_rowid() -> INTEGER
                            state.r.insert(
                                p3,
                                RegDataType::Single(ColumnType::Single {
                                    datatype: DataType::Int64,
                                    nullable: Some(false),
                                }),
                            );
                        }
                        "date(-1)" | "time(-1)" | "datetime(-1)" | "strftime(-1)" => {
                            // date|time|datetime|strftime(...) -> TEXT
                            state.r.insert(
                                p3,
                                RegDataType::Single(ColumnType::Single {
                                    datatype: DataType::Text,
                                    nullable: Some(p2 != 0), //never a null result if no argument provided
                                }),
                            );
                        }
                        "julianday(-1)" => {
                            // julianday(...) -> REAL
                            state.r.insert(
                                p3,
                                RegDataType::Single(ColumnType::Single {
                                    datatype: DataType::Float,
                                    nullable: Some(p2 != 0), //never a null result if no argument provided
                                }),
                            );
                        }
                        "unixepoch(-1)" => {
                            // unixepoch(p2...) -> INTEGER
                            state.r.insert(
                                p3,
                                RegDataType::Single(ColumnType::Single {
                                    datatype: DataType::Int64,
                                    nullable: Some(p2 != 0), //never a null result if no argument provided
                                }),
                            );
                        }

                        _ => logger.add_unknown_operation(&program[state.program_i]),
                    }
                }

                OP_NULL_ROW => {
                    // all columns in cursor X are potentially nullable
                    if let Some(CursorDataType::Normal { ref mut cols, .. }) = state.p.get_mut(&p1)
                    {
                        for col in cols.values_mut() {
                            if let ColumnType::Single {
                                ref mut nullable, ..
                            } = col
                            {
                                *nullable = Some(true);
                            }
                        }
                    }
                    //else we don't know about the cursor
                }

                OP_AGG_STEP | OP_AGG_VALUE => {
                    //assume that AGG_FINAL will be called
                    let p4 = from_utf8(p4).map_err(Error::protocol)?;

                    if p4.starts_with("count(")
                        || p4.starts_with("row_number(")
                        || p4.starts_with("rank(")
                        || p4.starts_with("dense_rank(")
                        || p4.starts_with("ntile(")
                    {
                        // count(_) -> INTEGER
                        state.r.insert(
                            p3,
                            RegDataType::Single(ColumnType::Single {
                                datatype: DataType::Int64,
                                nullable: Some(false),
                            }),
                        );
                    } else if p4.starts_with("sum(") {
                        if let Some(r_p2) = state.r.get(&p2) {
                            let datatype = match r_p2.map_to_datatype() {
                                DataType::Int64 => DataType::Int64,
                                DataType::Int => DataType::Int,
                                DataType::Bool => DataType::Int,
                                _ => DataType::Float,
                            };
                            let nullable = r_p2.map_to_nullable();
                            state.r.insert(
                                p3,
                                RegDataType::Single(ColumnType::Single { datatype, nullable }),
                            );
                        }
                    } else if let Some(v) = state.r.get(&p2).cloned() {
                        // r[p3] = AGG ( r[p2] )
                        state.r.insert(p3, v);
                    }
                }

                OP_AGG_FINAL => {
                    let p4 = from_utf8(p4).map_err(Error::protocol)?;

                    if p4.starts_with("count(")
                        || p4.starts_with("row_number(")
                        || p4.starts_with("rank(")
                        || p4.starts_with("dense_rank(")
                        || p4.starts_with("ntile(")
                    {
                        // count(_) -> INTEGER
                        state.r.insert(
                            p1,
                            RegDataType::Single(ColumnType::Single {
                                datatype: DataType::Int64,
                                nullable: Some(false),
                            }),
                        );
                    }
                }

                OP_CAST => {
                    // affinity(r[p1])
                    if let Some(v) = state.r.get_mut(&p1) {
                        *v = RegDataType::Single(ColumnType::Single {
                            datatype: affinity_to_type(p2 as u8),
                            nullable: v.map_to_nullable(),
                        });
                    }
                }

                OP_SCOPY | OP_INT_COPY => {
                    // r[p2] = r[p1]
                    if let Some(v) = state.r.get(&p1).cloned() {
                        state.r.insert(p2, v);
                    }
                }

                OP_COPY => {
                    // r[p2..=p2+p3] = r[p1..=p1+p3]
                    if p3 >= 0 {
                        for i in 0..=p3 {
                            let src = p1 + i;
                            let dst = p2 + i;
                            if let Some(v) = state.r.get(&src).cloned() {
                                state.r.insert(dst, v);
                            }
                        }
                    }
                }

                OP_MOVE => {
                    // r[p2..p2+p3] = r[p1..p1+p3]; r[p1..p1+p3] = null
                    if p3 >= 1 {
                        for i in 0..p3 {
                            let src = p1 + i;
                            let dst = p2 + i;
                            if let Some(v) = state.r.get(&src).cloned() {
                                state.r.insert(dst, v);
                                state.r.insert(src, RegDataType::Single(ColumnType::null()));
                            }
                        }
                    }
                }

                OP_INTEGER => {
                    // r[p2] = p1
                    state.r.insert(p2, RegDataType::Int(p1));
                }

                OP_BLOB | OP_COUNT | OP_REAL | OP_STRING8 | OP_ROWID | OP_NEWROWID => {
                    // r[p2] = <value of constant>
                    state.r.insert(
                        p2,
                        RegDataType::Single(ColumnType::Single {
                            datatype: opcode_to_type(&opcode),
                            nullable: Some(false),
                        }),
                    );
                }

                OP_NOT => {
                    // r[p2] = NOT r[p1]
                    if let Some(a) = state.r.get(&p1).cloned() {
                        state.r.insert(p2, a);
                    }
                }

                OP_NULL => {
                    // r[p2..p3] = null
                    let idx_range = if p2 < p3 { p2..=p3 } else { p2..=p2 };

                    for idx in idx_range {
                        state.r.insert(idx, RegDataType::Single(ColumnType::null()));
                    }
                }

                OP_OR | OP_AND | OP_BIT_AND | OP_BIT_OR | OP_SHIFT_LEFT | OP_SHIFT_RIGHT
                | OP_ADD | OP_SUBTRACT | OP_MULTIPLY | OP_DIVIDE | OP_REMAINDER | OP_CONCAT => {
                    // r[p3] = r[p1] + r[p2]
                    match (state.r.get(&p1).cloned(), state.r.get(&p2).cloned()) {
                        (Some(a), Some(b)) => {
                            state.r.insert(
                                p3,
                                RegDataType::Single(ColumnType::Single {
                                    datatype: if matches!(a.map_to_datatype(), DataType::Null) {
                                        b.map_to_datatype()
                                    } else {
                                        a.map_to_datatype()
                                    },
                                    nullable: match (a.map_to_nullable(), b.map_to_nullable()) {
                                        (Some(a_n), Some(b_n)) => Some(a_n | b_n),
                                        (Some(a_n), None) => Some(a_n),
                                        (None, Some(b_n)) => Some(b_n),
                                        (None, None) => None,
                                    },
                                }),
                            );
                        }

                        (Some(v), None) => {
                            state.r.insert(
                                p3,
                                RegDataType::Single(ColumnType::Single {
                                    datatype: v.map_to_datatype(),
                                    nullable: None,
                                }),
                            );
                        }

                        (None, Some(v)) => {
                            state.r.insert(
                                p3,
                                RegDataType::Single(ColumnType::Single {
                                    datatype: v.map_to_datatype(),
                                    nullable: None,
                                }),
                            );
                        }

                        _ => {}
                    }
                }

                OP_OFFSET_LIMIT => {
                    // r[p2] = if r[p2] < 0 { r[p1] } else if r[p1]<0 { -1 } else { r[p1] + r[p3] }
                    state.r.insert(
                        p2,
                        RegDataType::Single(ColumnType::Single {
                            datatype: DataType::Int64,
                            nullable: Some(false),
                        }),
                    );
                }

                OP_RESULT_ROW => {
                    // output = r[p1 .. p1 + p2]

                    state.result = Some(
                        (p1..p1 + p2)
                            .map(|i| {
                                let coltype = state.r.get(&i);

                                let sqltype =
                                    coltype.map(|d| d.map_to_datatype()).map(SqliteTypeInfo);
                                let nullable =
                                    coltype.map(|d| d.map_to_nullable()).unwrap_or_default();

                                (sqltype, nullable)
                            })
                            .collect(),
                    );

                    if logger.log_enabled() {
                        let program_history: Vec<&(i64, String, i64, i64, i64, Vec<u8>)> =
                            state.history.iter().map(|i| &program[*i]).collect();
                        logger.add_result((program_history, Some(state.result.clone())));
                    }

                    result_states.push(state.clone());
                }

                OP_HALT => {
                    if logger.log_enabled() {
                        let program_history: Vec<&(i64, String, i64, i64, i64, Vec<u8>)> =
                            state.history.iter().map(|i| &program[*i]).collect();
                        logger.add_result((program_history, None));
                    }
                    break;
                }

                _ => {
                    // ignore unsupported operations
                    // if we fail to find an r later, we just give up
                    logger.add_unknown_operation(&program[state.program_i]);
                }
            }

            state.program_i += 1;
        }
    }

    let mut output: Vec<Option<SqliteTypeInfo>> = Vec::new();
    let mut nullable: Vec<Option<bool>> = Vec::new();

    while let Some(state) = result_states.pop() {
        // find the datatype info from each ResultRow execution
        if let Some(result) = state.result {
            let mut idx = 0;
            for (this_type, this_nullable) in result {
                if output.len() == idx {
                    output.push(this_type);
                } else if output[idx].is_none()
                    || matches!(output[idx], Some(SqliteTypeInfo(DataType::Null)))
                {
                    output[idx] = this_type;
                }

                if nullable.len() == idx {
                    nullable.push(this_nullable);
                } else if let Some(ref mut null) = nullable[idx] {
                    //if any ResultRow's column is nullable, the final result is nullable
                    if let Some(this_null) = this_nullable {
                        *null |= this_null;
                    }
                } else {
                    nullable[idx] = this_nullable;
                }
                idx += 1;
            }
        }
    }

    let output = output
        .into_iter()
        .map(|o| o.unwrap_or(SqliteTypeInfo(DataType::Null)))
        .collect();

    Ok((output, nullable))
}

#[test]
fn test_root_block_columns_has_types() {
    use crate::SqliteConnectOptions;
    use std::str::FromStr;
    let conn_options = SqliteConnectOptions::from_str("sqlite::memory:").unwrap();
    let mut conn = super::EstablishParams::from_options(&conn_options)
        .unwrap()
        .establish()
        .unwrap();

    assert!(execute::iter(
        &mut conn,
        r"CREATE TABLE t(a INTEGER PRIMARY KEY, b_null TEXT NULL, b TEXT NOT NULL);",
        None,
        false
    )
    .unwrap()
    .next()
    .is_some());
    assert!(
        execute::iter(&mut conn, r"CREATE INDEX i1 on t (a,b_null);", None, false)
            .unwrap()
            .next()
            .is_some()
    );
    assert!(execute::iter(
        &mut conn,
        r"CREATE UNIQUE INDEX i2 on t (a,b_null);",
        None,
        false
    )
    .unwrap()
    .next()
    .is_some());
    assert!(execute::iter(
        &mut conn,
        r"CREATE TABLE t2(a INTEGER NOT NULL, b_null NUMERIC NULL, b NUMERIC NOT NULL);",
        None,
        false
    )
    .unwrap()
    .next()
    .is_some());
    assert!(execute::iter(
        &mut conn,
        r"CREATE INDEX t2i1 on t2 (a,b_null);",
        None,
        false
    )
    .unwrap()
    .next()
    .is_some());
    assert!(execute::iter(
        &mut conn,
        r"CREATE UNIQUE INDEX t2i2 on t2 (a,b);",
        None,
        false
    )
    .unwrap()
    .next()
    .is_some());

    assert!(execute::iter(
        &mut conn,
        r"CREATE TEMPORARY TABLE t3(a TEXT PRIMARY KEY, b REAL NOT NULL, b_null REAL NULL);",
        None,
        false
    )
    .unwrap()
    .next()
    .is_some());

    let table_block_nums: HashMap<String, (i64,i64)> = execute::iter(
        &mut conn,
        r"select name, 0 db_seq, rootpage from main.sqlite_schema UNION ALL select name, 1 db_seq, rootpage from temp.sqlite_schema",
        None,
        false,
    )
    .unwrap()
    .filter_map(|res| res.map(|either| either.right()).transpose())
    .map(|row| FromRow::from_row(row.as_ref().unwrap()))
    .map(|row| row.map(|(name,seq,block)|(name,(seq,block))))
    .collect::<Result<HashMap<_, _>, Error>>()
    .unwrap();

    let root_block_cols = root_block_columns(&mut conn).unwrap();

    // there should be 7 tables/indexes created explicitly, plus 1 autoindex for t3
    assert_eq!(8, root_block_cols.len());

    //prove that we have some information for each table & index
    for (name, db_seq_block) in dbg!(&table_block_nums) {
        assert!(
            root_block_cols.contains_key(db_seq_block),
            "{:?}",
            (name, db_seq_block)
        );
    }

    //prove that each block has the correct information
    {
        let table_db_block = table_block_nums["t"];
        assert_eq!(
            ColumnType::Single {
                datatype: DataType::Int64,
                nullable: Some(true) //sqlite primary key columns are nullable unless declared not null
            },
            root_block_cols[&table_db_block][&0]
        );
        assert_eq!(
            ColumnType::Single {
                datatype: DataType::Text,
                nullable: Some(true)
            },
            root_block_cols[&table_db_block][&1]
        );
        assert_eq!(
            ColumnType::Single {
                datatype: DataType::Text,
                nullable: Some(false)
            },
            root_block_cols[&table_db_block][&2]
        );
    }

    {
        let table_db_block = table_block_nums["i1"];
        assert_eq!(
            ColumnType::Single {
                datatype: DataType::Int64,
                nullable: Some(true) //sqlite primary key columns are nullable unless declared not null
            },
            root_block_cols[&table_db_block][&0]
        );
        assert_eq!(
            ColumnType::Single {
                datatype: DataType::Text,
                nullable: Some(true)
            },
            root_block_cols[&table_db_block][&1]
        );
    }

    {
        let table_db_block = table_block_nums["i2"];
        assert_eq!(
            ColumnType::Single {
                datatype: DataType::Int64,
                nullable: Some(true) //sqlite primary key columns are nullable unless declared not null
            },
            root_block_cols[&table_db_block][&0]
        );
        assert_eq!(
            ColumnType::Single {
                datatype: DataType::Text,
                nullable: Some(true)
            },
            root_block_cols[&table_db_block][&1]
        );
    }

    {
        let table_db_block = table_block_nums["t2"];
        assert_eq!(
            ColumnType::Single {
                datatype: DataType::Int64,
                nullable: Some(false)
            },
            root_block_cols[&table_db_block][&0]
        );
        assert_eq!(
            ColumnType::Single {
                datatype: DataType::Null,
                nullable: Some(true)
            },
            root_block_cols[&table_db_block][&1]
        );
        assert_eq!(
            ColumnType::Single {
                datatype: DataType::Null,
                nullable: Some(false)
            },
            root_block_cols[&table_db_block][&2]
        );
    }

    {
        let table_db_block = table_block_nums["t2i1"];
        assert_eq!(
            ColumnType::Single {
                datatype: DataType::Int64,
                nullable: Some(false)
            },
            root_block_cols[&table_db_block][&0]
        );
        assert_eq!(
            ColumnType::Single {
                datatype: DataType::Null,
                nullable: Some(true)
            },
            root_block_cols[&table_db_block][&1]
        );
    }

    {
        let table_db_block = table_block_nums["t2i2"];
        assert_eq!(
            ColumnType::Single {
                datatype: DataType::Int64,
                nullable: Some(false)
            },
            root_block_cols[&table_db_block][&0]
        );
        assert_eq!(
            ColumnType::Single {
                datatype: DataType::Null,
                nullable: Some(false)
            },
            root_block_cols[&table_db_block][&1]
        );
    }

    {
        let table_db_block = table_block_nums["t3"];
        assert_eq!(
            ColumnType::Single {
                datatype: DataType::Text,
                nullable: Some(true)
            },
            root_block_cols[&table_db_block][&0]
        );
        assert_eq!(
            ColumnType::Single {
                datatype: DataType::Float,
                nullable: Some(false)
            },
            root_block_cols[&table_db_block][&1]
        );
        assert_eq!(
            ColumnType::Single {
                datatype: DataType::Float,
                nullable: Some(true)
            },
            root_block_cols[&table_db_block][&2]
        );
    }
}
