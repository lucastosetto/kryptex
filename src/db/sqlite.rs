use rusqlite::{params, Connection, Result as SqliteResult};
use crate::models::signal::{SignalDirection, SignalOutput, SignalReason};

pub struct SignalDatabase {
    conn: Connection,
}

impl SignalDatabase {
    pub fn new(db_path: &str) -> SqliteResult<Self> {
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> SqliteResult<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS signals (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol TEXT NOT NULL,
                direction TEXT NOT NULL,
                confidence REAL NOT NULL,
                recommended_sl_pct REAL NOT NULL,
                recommended_tp_pct REAL NOT NULL,
                price REAL NOT NULL,
                timestamp TEXT NOT NULL,
                reasons_json TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn store_signal(&self, signal: &SignalOutput) -> SqliteResult<()> {
        let direction_str = match signal.direction {
            SignalDirection::Long => "Long",
            SignalDirection::Short => "Short",
            SignalDirection::Neutral => "Neutral",
        };

        let reasons_json = serde_json::to_string(&signal.reasons)
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

        self.conn.execute(
            "INSERT INTO signals (symbol, direction, confidence, recommended_sl_pct, recommended_tp_pct, price, timestamp, reasons_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                signal.symbol,
                direction_str,
                signal.confidence,
                signal.recommended_sl_pct,
                signal.recommended_tp_pct,
                signal.price,
                signal.timestamp.to_rfc3339(),
                reasons_json
            ],
        )?;
        Ok(())
    }

    pub fn get_all_signals(&self) -> SqliteResult<Vec<SignalOutput>> {
        let mut stmt = self.conn.prepare(
            "SELECT symbol, direction, confidence, recommended_sl_pct, recommended_tp_pct, price, timestamp, reasons_json
             FROM signals ORDER BY timestamp DESC"
        )?;

        let signal_iter = stmt.query_map([], |row| {
            let direction_str: String = row.get(1)?;
            let direction = match direction_str.as_str() {
                "Long" => SignalDirection::Long,
                "Short" => SignalDirection::Short,
                _ => SignalDirection::Neutral,
            };

            let reasons_json: String = row.get(7)?;
            let reasons: Vec<SignalReason> = serde_json::from_str(&reasons_json)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

            let timestamp_str: String = row.get(6)?;
            let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp_str)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?
                .with_timezone(&chrono::Utc);

            Ok(SignalOutput {
                symbol: row.get(0)?,
                direction,
                confidence: row.get(2)?,
                recommended_sl_pct: row.get(3)?,
                recommended_tp_pct: row.get(4)?,
                price: row.get(5)?,
                timestamp,
                reasons,
            })
        })?;

        let mut signals = Vec::new();
        for signal in signal_iter {
            signals.push(signal?);
        }
        Ok(signals)
    }

    pub fn get_signals_by_symbol(&self, symbol: &str) -> SqliteResult<Vec<SignalOutput>> {
        let mut stmt = self.conn.prepare(
            "SELECT symbol, direction, confidence, recommended_sl_pct, recommended_tp_pct, price, timestamp, reasons_json
             FROM signals WHERE symbol = ?1 ORDER BY timestamp DESC"
        )?;

        let signal_iter = stmt.query_map(params![symbol], |row| {
            let direction_str: String = row.get(1)?;
            let direction = match direction_str.as_str() {
                "Long" => SignalDirection::Long,
                "Short" => SignalDirection::Short,
                _ => SignalDirection::Neutral,
            };

            let reasons_json: String = row.get(7)?;
            let reasons: Vec<SignalReason> = serde_json::from_str(&reasons_json)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

            let timestamp_str: String = row.get(6)?;
            let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp_str)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?
                .with_timezone(&chrono::Utc);

            Ok(SignalOutput {
                symbol: row.get(0)?,
                direction,
                confidence: row.get(2)?,
                recommended_sl_pct: row.get(3)?,
                recommended_tp_pct: row.get(4)?,
                price: row.get(5)?,
                timestamp,
                reasons,
            })
        })?;

        let mut signals = Vec::new();
        for signal in signal_iter {
            signals.push(signal?);
        }
        Ok(signals)
    }

    pub fn get_signal_count(&self) -> SqliteResult<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM signals",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }
}

