// Persistent data import/export connections are defined within the OPTIONAL <data> tag, which contains one <import> tag for each data import connection and one <export> tag for each data export connection. Both tags include the following properties (the first four are optional):
// ·         Type:  type="…" with “CSV”, “Excel”, or “XML” (default: CSV)
// ·         Enabled state:  enabled="…" with true/false (default: true)
// ·         How often:  frequency="…" with either “on_demand” or “automatic” (default: automatic, i.e., whenever the data changes)
// ·         Data orientation:  orientation="…" with either “horizontal” or “vertical” (default: vertical)
// ·         Source (import) or destination (export) location:  resource="…".  A resource can be a relative file path, an absolute file path, or an URL.
// ·         For Excel only, worksheet name:  worksheet="…" with worksheet name
// The <export> also specifies both the optional export interval and one of two sources of the data:
// ·         Export interval:  interval="…" specifying how often, in model time, to export values during the simulation; use "DT" to export every DT (default: 0, meaning only once)
// ·         <all/> to export all variables in the whole-model or <table uid="…"/> to just export the variables named in the table (note that any array element in the table will export the entire array when interval is set to zero). The <table> tag has an optional attribute use_settings="…" with a true/false value (default: false), which when true causes the table settings for orientation, interval, and number formatting to be used (thus, when it is set, neither orientation nor interval are meaningful, so should not appear). The uid used for the table must be qualified by the name of the module in which the table appears.  If in the root a ‘.’ is prefixed to the name, same as module qualified variable names.

pub struct Data {
    /// A list of data import connections in the XMILE file.
    pub imports: Vec<DataImport>,
    /// A list of data export connections in the XMILE file.
    pub exports: Vec<DataExport>,
}

pub struct DataImport {
    /// The type of the data import (e.g., CSV, Excel, XML).
    pub data_type: Option<String>,
    /// Indicates whether the data import is enabled.
    pub enabled: Option<bool>,
    /// The frequency of the data import (e.g., on_demand, automatic).
    pub frequency: Option<String>,
    /// The orientation of the data import (e.g., horizontal, vertical).
    pub orientation: Option<String>,
    /// The source location of the data import.
    pub resource: Option<String>,
    /// The worksheet name for Excel imports.
    pub worksheet: Option<String>,
}

pub struct DataExport {
    /// The type of the data export (e.g., CSV, Excel, XML).
    pub data_type: Option<String>,
    /// Indicates whether the data export is enabled.
    pub enabled: Option<bool>,
    /// The frequency of the data export (e.g., on_demand, automatic).
    pub frequency: Option<String>,
    /// The orientation of the data export (e.g., horizontal, vertical).
    pub orientation: Option<String>,
    /// The destination location of the data export.
    pub resource: Option<String>,
    /// The worksheet name for Excel exports.
    pub worksheet: Option<String>,
    /// The export interval in model time.
    pub interval: Option<String>,
    /// Indicates whether to export all variables or a specific table.
    pub export_all: bool,
    /// The UID of the table to export (if not exporting all variables).
    pub table_uid: Option<String>,
}
