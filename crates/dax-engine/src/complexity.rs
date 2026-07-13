// TRRUSTT — DAX Complexity Levels
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComplexityLevel {
    Beginner = 0,
    Intermediate = 1,
    Advanced = 2,
    Expert = 3,
}

impl ComplexityLevel {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "beginner" => Some(Self::Beginner),
            "intermediate" => Some(Self::Intermediate),
            "advanced" => Some(Self::Advanced),
            "expert" => Some(Self::Expert),
            _ => None,
        }
    }

    /// Get allowed DAX functions for this complexity level.
    pub fn allowed_functions(&self) -> &[&str] {
        match self {
            Self::Beginner => &["SUM", "COUNT", "AVERAGE", "MIN", "MAX", "DISTINCTCOUNT", "COUNTROWS", "IF", "BLANK", "DIVIDE"],
            Self::Intermediate => &["SUM", "COUNT", "AVERAGE", "MIN", "MAX", "DISTINCTCOUNT", "COUNTROWS", "IF", "BLANK", "DIVIDE", "CALCULATE", "FILTER", "ALL", "ALLEXCEPT", "ALLSELECTED", "VALUES", "HASONEVALUE", "SELECTEDVALUE", "RELATED", "RELATEDTABLE", "USERELATIONSHIP", "SAMEPERIODLASTYEAR", "DATEADD", "DATESYTD", "DATESQTD", "DATESMTD", "TOTALYTD", "TOTALQTD", "TOTALMTD", "VAR", "RETURN"],
            Self::Advanced => &["SUM", "COUNT", "AVERAGE", "MIN", "MAX", "DISTINCTCOUNT", "COUNTROWS", "IF", "SWITCH", "BLANK", "DIVIDE", "CALCULATE", "FILTER", "ALL", "ALLEXCEPT", "ALLSELECTED", "KEEPFILTERS", "REMOVEFILTERS", "VALUES", "HASONEVALUE", "SELECTEDVALUE", "RELATED", "RELATEDTABLE", "USERELATIONSHIP", "CROSSFILTER", "SAMEPERIODLASTYEAR", "DATEADD", "DATESYTD", "DATESQTD", "DATESMTD", "TOTALYTD", "TOTALQTD", "TOTALMTD", "PARALLELPERIOD", "PREVIOUSMONTH", "PREVIOUSQUARTER", "PREVIOUSYEAR", "VAR", "RETURN", "CALCULATETABLE", "SUMMARIZE", "ADDCOLUMNS", "TOPN", "RANKX", "SUMX", "AVERAGEX", "MAXX", "MINX", "COUNTX"],
            Self::Expert => &[],
        }
    }
}

impl fmt::Display for ComplexityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self { Self::Beginner => "beginner", Self::Intermediate => "intermediate", Self::Advanced => "advanced", Self::Expert => "expert" };
        write!(f, "{}", s)
    }
}
    Advanced,
    Expert,
}
