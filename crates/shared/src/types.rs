// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Shared Domain Types
//
// All core domain types used across crates. Serializable/deserializable
// via serde. These are the canonical representations of all business
// entities in the system.
// ═══════════════════════════════════════════════════════════════════════

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════════════
// IDENTITY & ACCESS
// ═══════════════════════════════════════════════════════════════════════

/// A user of the TRRUSTT system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    /// Unique identifier (UUID v7).
    pub id: Uuid,
    /// Primary email address.
    pub email: String,
    /// Display name.
    pub display_name: String,
    /// Optional avatar URL.
    pub avatar_url: Option<String>,
    /// Authentication provider.
    pub auth_provider: AuthProvider,
    /// External user ID from the auth provider.
    pub auth_provider_user_id: Option<String>,
    /// SSO tenant/domain.
    pub sso_tenant_id: Option<String>,
    /// PBI Desktop user principal name (auto-detected).
    pub pbi_desktop_user_principal: Option<String>,
    /// User's global role.
    pub role: UserRole,
    /// Whether the account is active.
    pub is_active: bool,
    /// Arbitrary user preferences (JSON).
    pub preferences: serde_json::Value,
    /// When the account was created.
    pub created_at: DateTime<Utc>,
    /// When the account was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Authentication provider for a user account.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AuthProvider {
    /// Local password-based auth.
    Local,
    /// Microsoft Entra ID / Azure AD.
    AzureAd,
    /// Google Workspace / Google Identity.
    Google,
    /// Generic OpenID Connect provider.
    Oidc,
    /// Auto-detected from Power BI Desktop session.
    PbiDesktop,
}

/// Global user role (highest to lowest privilege).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    /// Can only view dashboards.
    Viewer = 0,
    /// Can explore data and run analyses.
    Analyst = 1,
    /// Can create dashboards and measures.
    Designer = 2,
    /// Can manage workspace, users, and settings.
    Admin = 3,
    /// Full system access, can override policies.
    SuperAdmin = 4,
}

impl UserRole {
    /// Check if this role has at least the required privilege level.
    pub fn can(&self, required: UserRole) -> bool {
        *self >= required
    }

    /// Parse from a string (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "viewer" => Some(Self::Viewer),
            "analyst" => Some(Self::Analyst),
            "designer" => Some(Self::Designer),
            "admin" => Some(Self::Admin),
            "super_admin" | "superadmin" => Some(Self::SuperAdmin),
            _ => None,
        }
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            UserRole::Viewer => "viewer",
            UserRole::Analyst => "analyst",
            UserRole::Designer => "designer",
            UserRole::Admin => "admin",
            UserRole::SuperAdmin => "super_admin",
        };
        write!(f, "{}", s)
    }
}

/// An organization (tenant, company, team).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Organization {
    /// Unique identifier.
    pub id: Uuid,
    /// Display name.
    pub name: String,
    /// URL-friendly identifier.
    pub slug: String,
    /// Subscription plan tier.
    pub plan_tier: PlanTier,
    /// When the license expires (None = perpetual).
    pub license_expires_at: Option<DateTime<Utc>>,
    /// Maximum number of users (None = unlimited).
    pub max_users: Option<i32>,
    /// Maximum number of workspaces.
    pub max_workspaces: i32,
    /// Organization-wide settings (JSON).
    pub settings: serde_json::Value,
    /// White-label branding config (OEM only).
    pub branding: Option<serde_json::Value>,
    /// SSO domain for enterprise.
    pub sso_domain: Option<String>,
    /// SSO provider type.
    pub sso_provider: Option<SsoProvider>,
    /// When the org was created.
    pub created_at: DateTime<Utc>,
    /// When the org was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Subscription plan tier.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PlanTier {
    /// Free tier.
    Free,
    /// Pro tier ($149/yr).
    Pro,
    /// Team tier ($49/user/mo).
    Team,
    /// Enterprise tier.
    Enterprise,
    /// OEM / white-label tier.
    Oem,
}

/// SSO provider type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SsoProvider {
    /// Microsoft Entra ID / Azure AD.
    AzureAd,
    /// Google Workspace.
    GoogleWorkspace,
    /// Okta.
    Okta,
    /// Generic OpenID Connect.
    GenericOidc,
}

/// Membership of a user in an organization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationMember {
    /// Organization ID.
    pub org_id: Uuid,
    /// User ID.
    pub user_id: Uuid,
    /// Role within the organization.
    pub org_role: OrgRole,
    /// When the user joined.
    pub joined_at: DateTime<Utc>,
}

/// Role within an organization.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrgRole {
    /// Organization owner.
    Owner,
    /// Organization admin.
    Admin,
    /// Regular member.
    Member,
}

// ═══════════════════════════════════════════════════════════════════════
// WORKSPACES & COLLABORATION
// ═══════════════════════════════════════════════════════════════════════

/// A workspace groups projects and settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Workspace {
    /// Unique identifier.
    pub id: Uuid,
    /// Owning organization.
    pub org_id: Uuid,
    /// Workspace name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Workspace-level settings (JSON).
    pub settings: serde_json::Value,
    /// Whether this is the default workspace for the org.
    pub is_default: bool,
    /// Who created this workspace.
    pub created_by: Uuid,
    /// When created.
    pub created_at: DateTime<Utc>,
    /// When last updated.
    pub updated_at: DateTime<Utc>,
}

/// Membership of a user in a workspace with RBAC.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceMember {
    /// Workspace ID.
    pub workspace_id: Uuid,
    /// User ID.
    pub user_id: Uuid,
    /// Role within the workspace.
    pub workspace_role: UserRole,
    /// Fine-grained permissions override (JSON).
    pub permissions: Option<serde_json::Value>,
    /// When the user joined.
    pub joined_at: DateTime<Utc>,
}

/// An invitation to join a workspace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceInvitation {
    /// Unique identifier.
    pub id: Uuid,
    /// Target workspace.
    pub workspace_id: Uuid,
    /// Invited email address.
    pub email: String,
    /// Role to assign upon acceptance.
    pub invited_role: UserRole,
    /// Who sent the invitation.
    pub invited_by: Uuid,
    /// Unique invitation token.
    pub token: String,
    /// Current status.
    pub status: InvitationStatus,
    /// When the invitation expires.
    pub expires_at: DateTime<Utc>,
    /// When created.
    pub created_at: DateTime<Utc>,
}

/// Status of a workspace invitation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InvitationStatus {
    /// Awaiting response.
    Pending,
    /// Accepted by the user.
    Accepted,
    /// Expired without response.
    Expired,
    /// Revoked by an admin.
    Revoked,
}

// ═══════════════════════════════════════════════════════════════════════
// PROJECTS & POWER BI
// ═══════════════════════════════════════════════════════════════════════

/// A project represents one .pbix file / dataset.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    /// Unique identifier.
    pub id: Uuid,
    /// Parent workspace.
    pub workspace_id: Uuid,
    /// Project name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Path to the .pbix file.
    pub pbix_path: Option<String>,
    /// SSAS instance port.
    pub ssas_port: Option<u16>,
    /// SSAS database name.
    pub ssas_database: Option<String>,
    /// SHA-256 hash of the last discovered schema.
    pub schema_hash: Option<String>,
    /// Full schema snapshot (JSON).
    pub schema_snapshot: Option<serde_json::Value>,
    /// When the schema was last discovered.
    pub schema_discovered_at: Option<DateTime<Utc>>,
    /// Default DAX complexity for this project.
    pub dax_complexity_default: DaxComplexity,
    /// DAX naming convention for this project.
    pub dax_naming_convention: Option<String>,
    /// PBI Desktop version string.
    pub pbi_desktop_version: Option<String>,
    /// PBI Desktop culture (e.g., "en-US").
    pub pbi_desktop_culture: Option<String>,
    /// Count of measures in this project.
    pub measure_count: i32,
    /// Count of dashboards in this project.
    pub dashboard_count: i32,
    /// When the project was last accessed.
    pub last_accessed_at: Option<DateTime<Utc>>,
    /// Who created the project.
    pub created_by: Uuid,
    /// When created.
    pub created_at: DateTime<Utc>,
    /// When last updated.
    pub updated_at: DateTime<Utc>,
}

/// DAX complexity level.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum DaxComplexity {
    /// Basic aggregations: SUM, COUNT, AVERAGE, etc.
    Beginner = 0,
    /// CALCULATE, FILTER, ALL, time intelligence basics.
    Intermediate = 1,
    /// Complex CALCULATE modifiers, advanced time intelligence, variables.
    Advanced = 2,
    /// Full DAX: complex iterators, evaluation contexts, query plans.
    Expert = 3,
}

impl DaxComplexity {
    /// Parse from a string (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "beginner" => Some(Self::Beginner),
            "intermediate" => Some(Self::Intermediate),
            "advanced" => Some(Self::Advanced),
            "expert" => Some(Self::Expert),
            _ => None,
        }
    }
}

impl std::fmt::Display for DaxComplexity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DaxComplexity::Beginner => "beginner",
            DaxComplexity::Intermediate => "intermediate",
            DaxComplexity::Advanced => "advanced",
            DaxComplexity::Expert => "expert",
        };
        write!(f, "{}", s)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SCHEMA METADATA (Power BI Data Model)
// ═══════════════════════════════════════════════════════════════════════

/// Complete schema metadata for a Power BI data model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaMetadata {
    /// Model name (database name in SSAS).
    pub name: String,
    /// Model description.
    pub description: Option<String>,
    /// All tables in the model.
    pub tables: Vec<TableInfo>,
    /// All relationships in the model.
    pub relationships: Vec<RelationshipInfo>,
    /// All measures in the model.
    pub measures: Vec<MeasureInfo>,
    /// All calculation groups.
    pub calculation_groups: Vec<CalculationGroupInfo>,
    /// Model-level annotations.
    pub annotations: Vec<Annotation>,
    /// When this schema was discovered.
    pub discovered_at: DateTime<Utc>,
    /// SSAS compatibility level.
    pub compatibility_level: i32,
    /// Model default mode (Import, DirectQuery, Dual, etc.).
    pub default_mode: Option<String>,
}

/// Information about a table in the model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableInfo {
    /// Table name (as it appears in the model).
    pub name: String,
    /// Whether this is a hidden table.
    pub is_hidden: bool,
    /// Whether this is a calculated table.
    pub is_calculated: bool,
    /// Storage mode: Import, DirectQuery, Dual, etc.
    pub storage_mode: Option<String>,
    /// Columns in this table.
    pub columns: Vec<ColumnInfo>,
    /// Hierarchies in this table.
    pub hierarchies: Vec<HierarchyInfo>,
    /// Partitions in this table.
    pub partitions: Vec<PartitionInfo>,
    /// Table-level annotations.
    pub annotations: Vec<Annotation>,
    /// Row count estimate.
    pub row_count_estimate: Option<i64>,
}

/// Information about a column in a table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColumnInfo {
    /// Column name.
    pub name: String,
    /// Data type.
    pub data_type: ColumnDataType,
    /// Whether this is a hidden column.
    pub is_hidden: bool,
    /// Whether this is a calculated column.
    pub is_calculated: bool,
    /// Whether this is a key column.
    pub is_key: bool,
    /// Whether this column allows nulls.
    pub is_nullable: bool,
    /// The DAX expression if this is a calculated column.
    pub expression: Option<String>,
    /// Format string for display.
    pub format_string: Option<String>,
    /// Sort-by column name.
    pub sort_by_column: Option<String>,
    /// Column-level annotations.
    pub annotations: Vec<Annotation>,
    /// Distinct value count estimate.
    pub distinct_count_estimate: Option<i64>,
}

/// Column data types in Power BI / SSAS Tabular.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ColumnDataType {
    /// 64-bit integer.
    Int64,
    /// 64-bit floating point.
    Double,
    /// Text string.
    String,
    /// Boolean (true/false).
    Boolean,
    /// Date and time.
    DateTime,
    /// Currency / fixed decimal.
    Currency,
    /// Binary data.
    Binary,
    /// Variant (can hold any type).
    Variant,
}

/// Information about a measure in the model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MeasureInfo {
    /// Measure name.
    pub name: String,
    /// Parent table name.
    pub table_name: String,
    /// DAX expression.
    pub expression: String,
    /// Format string.
    pub format_string: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Display folder in the field list.
    pub display_folder: Option<String>,
    /// Data type.
    pub data_type: ColumnDataType,
    /// Whether this is hidden.
    pub is_hidden: bool,
    /// Measure-level annotations.
    pub annotations: Vec<Annotation>,
}

/// Information about a relationship between tables.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelationshipInfo {
    /// Relationship name.
    pub name: String,
    /// From table.
    pub from_table: String,
    /// From column.
    pub from_column: String,
    /// To table.
    pub to_table: String,
    /// To column.
    pub to_column: String,
    /// Cardinality: ManyToOne, OneToMany, OneToOne.
    pub cardinality: RelationshipCardinality,
    /// Cross filtering direction.
    pub cross_filter_direction: CrossFilterDirection,
    /// Whether this relationship is active.
    pub is_active: bool,
    /// Security filtering behavior.
    pub security_filtering: bool,
}

/// Relationship cardinality.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum RelationshipCardinality {
    /// Many-to-one.
    ManyToOne,
    /// One-to-many.
    OneToMany,
    /// One-to-one.
    OneToOne,
}

/// Cross-filter direction for relationships.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum CrossFilterDirection {
    /// Single direction (from one side to many).
    Single,
    /// Both directions.
    Both,
}

/// Information about a hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HierarchyInfo {
    /// Hierarchy name.
    pub name: String,
    /// Parent table name.
    pub table_name: String,
    /// Levels in the hierarchy (ordered).
    pub levels: Vec<HierarchyLevel>,
    /// Whether this hierarchy is hidden.
    pub is_hidden: bool,
}

/// A level in a hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HierarchyLevel {
    /// Level name.
    pub name: String,
    /// Source column name.
    pub source_column: String,
    /// Ordinal position (0-based).
    pub ordinal: i32,
}

/// Information about a calculation group.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalculationGroupInfo {
    /// Calculation group name.
    pub name: String,
    /// Calculation items.
    pub items: Vec<CalculationItem>,
    /// Precedence.
    pub precedence: i32,
}

/// A calculation item within a calculation group.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalculationItem {
    /// Item name.
    pub name: String,
    /// DAX expression.
    pub expression: String,
    /// Ordinal position.
    pub ordinal: i32,
}

/// Information about a partition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartitionInfo {
    /// Partition name.
    pub name: String,
    /// Query definition (M or SQL).
    pub query_definition: Option<String>,
    /// Partition mode: Import, DirectQuery, Dual, etc.
    pub mode: Option<String>,
}

/// A generic annotation (key-value metadata).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Annotation {
    /// Annotation name.
    pub name: String,
    /// Annotation value.
    pub value: String,
}

// ═══════════════════════════════════════════════════════════════════════
// DAX MEASURES
// ═══════════════════════════════════════════════════════════════════════

/// A generated or user-created DAX measure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Measure {
    /// Unique identifier.
    pub id: Uuid,
    /// Parent project.
    pub project_id: Uuid,
    /// Measure name.
    pub name: String,
    /// Parent table name.
    pub table_name: String,
    /// DAX expression.
    pub expression: String,
    /// Format string.
    pub format_string: Option<String>,
    /// Human-readable description.
    pub description: Option<String>,
    /// Display folder.
    pub display_folder: Option<String>,
    /// Data type.
    pub data_type: ColumnDataType,
    /// Complexity level.
    pub complexity: DaxComplexity,
    /// Whether applied to the SSAS model.
    pub is_applied: bool,
    /// Whether generated by AI.
    pub is_ai_generated: bool,
    /// The prompt used to generate this.
    pub ai_prompt_used: Option<String>,
    /// Which AI provider generated it.
    pub ai_provider: Option<String>,
    /// Which AI model generated it.
    pub ai_model: Option<String>,
    /// Last validation status.
    pub validation_status: Option<ValidationStatus>,
    /// Last validation errors (JSON).
    pub validation_errors: Option<serde_json::Value>,
    /// Parent measure (for versioning).
    pub parent_measure_id: Option<Uuid>,
    /// Version number.
    pub version: i32,
    /// Who created this.
    pub created_by: Option<Uuid>,
    /// When created.
    pub created_at: DateTime<Utc>,
    /// When last updated.
    pub updated_at: DateTime<Utc>,
}

/// Validation status of a measure.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ValidationStatus {
    /// All checks passed.
    Valid,
    /// Has warnings (non-blocking).
    Warning,
    /// Has errors (blocking).
    Error,
    /// Was corrected by the self-correction loop.
    Corrected,
}

// ═══════════════════════════════════════════════════════════════════════
// DASHBOARDS
// ═══════════════════════════════════════════════════════════════════════

/// A dashboard with pages and visuals.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Dashboard {
    /// Unique identifier.
    pub id: Uuid,
    /// Parent project.
    pub project_id: Uuid,
    /// Dashboard name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// All pages with their visuals (JSON).
    pub pages: serde_json::Value,
    /// Layout configuration (JSON).
    pub layout_config: Option<serde_json::Value>,
    /// Original natural language intent.
    pub user_intent: Option<String>,
    /// Source image path (if created from image).
    pub image_source_path: Option<String>,
    /// Applied theme ID.
    pub theme_id: Option<Uuid>,
    /// Number of pages.
    pub page_count: i32,
    /// Number of visuals.
    pub visual_count: i32,
    /// Number of measures.
    pub measure_count: i32,
    /// Version number.
    pub version: i32,
    /// Parent dashboard for versioning.
    pub parent_dashboard_id: Option<Uuid>,
    /// Who created this.
    pub created_by: Option<Uuid>,
    /// When created.
    pub created_at: DateTime<Utc>,
    /// When last updated.
    pub updated_at: DateTime<Utc>,
}

/// A single page within a dashboard.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DashboardPage {
    /// Page name / title.
    pub name: String,
    /// Page display name.
    pub display_name: Option<String>,
    /// Visuals on this page.
    pub visuals: Vec<DashboardVisual>,
    /// Page-level layout configuration.
    pub layout: Option<PageLayout>,
    /// Whether this page is hidden.
    pub is_hidden: bool,
    /// Page width.
    pub width: Option<i32>,
    /// Page height.
    pub height: Option<i32>,
}

/// A single visual on a dashboard page.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DashboardVisual {
    /// Visual name.
    pub name: String,
    /// Visual type (barChart, lineChart, card, etc.).
    pub visual_type: VisualType,
    /// Position on the canvas.
    pub position: VisualPosition,
    /// Size on the canvas.
    pub size: VisualSize,
    /// Field bindings (measures/columns bound to visual slots).
    pub field_bindings: serde_json::Value,
    /// Visual-level format configuration.
    pub format_config: Option<serde_json::Value>,
    /// Title text override.
    pub title: Option<String>,
}

/// Type of Power BI visual.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum VisualType {
    /// Stacked or clustered bar chart.
    BarChart,
    /// Stacked or clustered column chart.
    ColumnChart,
    /// Line chart.
    LineChart,
    /// Area chart.
    AreaChart,
    /// Pie chart.
    PieChart,
    /// Donut chart.
    DonutChart,
    /// Treemap.
    Treemap,
    /// Single-value card.
    Card,
    /// Multi-row card.
    MultiRowCard,
    /// KPI visual.
    Kpi,
    /// Table visual.
    Table,
    /// Matrix visual.
    Matrix,
    /// Scatter chart.
    ScatterChart,
    /// Waterfall chart.
    WaterfallChart,
    /// Funnel chart.
    FunnelChart,
    /// Gauge visual.
    Gauge,
    /// Slicer.
    Slicer,
    /// Map visual.
    Map,
    /// Decomposition tree.
    DecompositionTree,
}

/// Position of a visual on the canvas (x, y, z-order).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisualPosition {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
    /// Z-order (stacking order).
    pub z: i32,
}

/// Size of a visual on the canvas (width, height).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisualSize {
    /// Width in pixels.
    pub width: f64,
    /// Height in pixels.
    pub height: f64,
}

/// Page-level layout configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PageLayout {
    /// Number of grid columns.
    pub grid_columns: i32,
    /// Grid density.
    pub density: GridDensity,
    /// Padding between visuals.
    pub padding: f64,
}

/// Grid density for dashboard layouts.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GridDensity {
    /// More whitespace, fewer visuals.
    Compact,
    /// Balanced.
    Medium,
    /// Less whitespace, more visuals.
    Spacious,
}

// ═══════════════════════════════════════════════════════════════════════
// THEMES
// ═══════════════════════════════════════════════════════════════════════

/// A dashboard theme.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    /// Unique identifier.
    pub id: Uuid,
    /// Theme name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Whether this is a built-in default theme.
    pub is_default: bool,
    /// Color palette.
    pub colors: ThemeColors,
    /// Typography settings.
    pub typography: ThemeTypography,
    /// Visual defaults.
    pub visual_defaults: serde_json::Value,
    /// When created.
    pub created_at: DateTime<Utc>,
    /// When last updated.
    pub updated_at: DateTime<Utc>,
}

/// Color palette for a theme.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeColors {
    /// Primary color.
    pub primary: String,
    /// Secondary color.
    pub secondary: String,
    /// Accent color.
    pub accent: String,
    /// Background color.
    pub background: String,
    /// Foreground / text color.
    pub foreground: String,
    /// Data colors (for chart series).
    pub data_colors: Vec<String>,
    /// Semantic colors (good, bad, neutral, warning).
    pub semantic: SemanticColors,
}

/// Semantic colors for data visualization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticColors {
    /// Positive / good (green).
    pub good: String,
    /// Negative / bad (red).
    pub bad: String,
    /// Neutral (gray).
    pub neutral: String,
    /// Warning (yellow/orange).
    pub warning: String,
}

/// Typography settings for a theme.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThemeTypography {
    /// Primary font family.
    pub font_family: String,
    /// Title font size.
    pub title_size: i32,
    /// Body font size.
    pub body_size: i32,
    /// Label font size.
    pub label_size: i32,
    /// KPI value font size.
    pub kpi_size: i32,
}

// ═══════════════════════════════════════════════════════════════════════
// AI USAGE & CACHE
// ═══════════════════════════════════════════════════════════════════════

/// A record of an AI API call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AiUsage {
    /// Unique identifier.
    pub id: Uuid,
    /// Associated project.
    pub project_id: Option<Uuid>,
    /// User who made the call.
    pub user_id: Uuid,
    /// AI provider used.
    pub provider: String,
    /// Model used.
    pub model: String,
    /// Operation type (chat, dax_generate, dax_validate, etc.).
    pub operation: String,
    /// Input tokens used.
    pub input_tokens: i64,
    /// Output tokens used.
    pub output_tokens: i64,
    /// Total tokens used.
    pub total_tokens: i64,
    /// Cost in USD.
    pub cost_usd: f64,
    /// Whether this was a cache hit.
    pub cache_hit: bool,
    /// Duration in milliseconds.
    pub duration_ms: i64,
    /// When the call was made.
    pub created_at: DateTime<Utc>,
}

/// A cached AI response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiCacheEntry {
    /// Cache key (hash of prompt + parameters).
    pub cache_key: String,
    /// Hash of the prompt for lookup.
    pub prompt_hash: String,
    /// The cached response.
    pub response: String,
    /// AI provider used.
    pub provider: String,
    /// Model used.
    pub model: String,
    /// Operation type.
    pub operation: String,
    /// When this entry was created.
    pub created_at: DateTime<Utc>,
    /// When this entry expires.
    pub expires_at: DateTime<Utc>,
}

// ═══════════════════════════════════════════════════════════════════════
// AUDIT LOG
// ═══════════════════════════════════════════════════════════════════════

/// An audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditLogEntry {
    /// Unique identifier.
    pub id: Uuid,
    /// User who performed the action.
    pub user_id: Option<Uuid>,
    /// Organization context.
    pub org_id: Option<Uuid>,
    /// Workspace context.
    pub workspace_id: Option<Uuid>,
    /// Project context.
    pub project_id: Option<Uuid>,
    /// The action performed.
    pub action: String,
    /// Target entity type.
    pub entity_type: Option<String>,
    /// Target entity ID.
    pub entity_id: Option<Uuid>,
    /// Action details (JSON).
    pub details: Option<serde_json::Value>,
    /// IP address of the user.
    pub ip_address: Option<String>,
    /// User agent string.
    pub user_agent: Option<String>,
    /// When the action occurred.
    pub created_at: DateTime<Utc>,
}

// ═══════════════════════════════════════════════════════════════════════
// CONFIG & POLICIES
// ═══════════════════════════════════════════════════════════════════════

/// A configuration change history entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigHistoryEntry {
    /// Unique identifier.
    pub id: Uuid,
    /// Which config key was changed.
    pub config_key: String,
    /// Previous value (JSON).
    pub previous_value: Option<String>,
    /// New value (JSON).
    pub new_value: String,
    /// Which scope was modified.
    pub scope: String,
    /// Who made the change.
    pub changed_by: Option<Uuid>,
    /// When the change was made.
    pub changed_at: DateTime<Utc>,
}

/// A policy definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Policy {
    /// Unique policy identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Policy category.
    pub category: PolicyCategory,
    /// The enforced value (JSON).
    pub value: serde_json::Value,
    /// Default value.
    pub default_value: serde_json::Value,
    /// Who can override this policy.
    pub overridable_by: Vec<UserRole>,
    /// Whether this policy is enabled.
    pub enabled: bool,
    /// Rationale/documentation.
    pub rationale: String,
    /// Who set this policy.
    pub set_by: Option<String>,
    /// When it was set.
    pub set_at: Option<DateTime<Utc>>,
}

/// Policy category enum.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyCategory {
    /// Backup & recovery.
    Backup,
    /// Data retention & lifecycle.
    DataRetention,
    /// Security & access control.
    Security,
    /// AI usage & cost management.
    AiUsage,
    /// DAX generation rules.
    DaxGovernance,
    /// API & MCP rate limiting.
    RateLimit,
    /// Notification & alerting.
    Notification,
    /// Compliance (GDPR, SOC2).
    Compliance,
    /// Feature access (license-tier enforcement).
    FeatureAccess,
}

// ═══════════════════════════════════════════════════════════════════════
// LICENSE
// ═══════════════════════════════════════════════════════════════════════

/// Decoded license information from a JWT.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LicenseInfo {
    /// Licensee name (individual or company).
    pub licensee: String,
    /// Licensee email.
    pub email: String,
    /// Licensed tier.
    pub tier: PlanTier,
    /// Maximum seats (for team/enterprise).
    pub seats: Option<i32>,
    /// Enabled feature flags.
    pub features: Vec<String>,
    /// License issue date.
    pub issued_at: DateTime<Utc>,
    /// License expiration date.
    pub expires_at: DateTime<Utc>,
    /// Unique license ID.
    pub license_id: String,
}

// ═══════════════════════════════════════════════════════════════════════
// MCP
// ═══════════════════════════════════════════════════════════════════════

/// An MCP tool definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpTool {
    /// Tool name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// JSON Schema for input parameters.
    pub input_schema: serde_json::Value,
}

/// An MCP tool execution result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpToolResult {
    /// Whether execution succeeded.
    pub success: bool,
    /// Result content (text or JSON).
    pub content: serde_json::Value,
    /// Error message if failed.
    pub error: Option<String>,
    /// Duration in milliseconds.
    pub duration_ms: u64,
}

// ═══════════════════════════════════════════════════════════════════════
// XMLA
// ═══════════════════════════════════════════════════════════════════════

/// Result from a DAX query execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DaxQueryResult {
    /// Column names in result order.
    pub columns: Vec<String>,
    /// Row data (each row is an array of JSON values).
    pub rows: Vec<Vec<serde_json::Value>>,
    /// Total row count.
    pub row_count: usize,
    /// Query duration in milliseconds.
    pub duration_ms: u64,
}

/// Result from a TMSL command execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TmslResult {
    /// Whether the command succeeded.
    pub success: bool,
    /// Affected objects (e.g., measures created/deleted).
    pub affected_objects: Vec<String>,
    /// Any warnings returned by SSAS.
    pub warnings: Vec<String>,
    /// Duration in milliseconds.
    pub duration_ms: u64,
}

// ═══════════════════════════════════════════════════════════════════════
// AI REQUEST/RESPONSE
// ═══════════════════════════════════════════════════════════════════════

/// A request to an AI provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatRequest {
    /// System prompt.
    pub system_prompt: Option<String>,
    /// User message.
    pub user_message: String,
    /// Conversation history.
    pub history: Vec<ChatMessage>,
    /// Target provider (overrides default).
    pub provider: Option<String>,
    /// Target model (overrides default).
    pub model: Option<String>,
    /// Temperature (0.0 - 2.0).
    pub temperature: Option<f64>,
    /// Maximum output tokens.
    pub max_tokens: Option<u32>,
    /// JSON schema for structured output.
    pub response_format: Option<serde_json::Value>,
    /// Additional provider-specific parameters.
    pub extra_params: Option<serde_json::Value>,
}

/// A chat message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    /// Role: system, user, assistant.
    pub role: String,
    /// Message content.
    pub content: String,
}

/// A response from an AI provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatResponse {
    /// The assistant's response text.
    pub content: String,
    /// Which provider was used.
    pub provider: String,
    /// Which model was used.
    pub model: String,
    /// Token usage.
    pub tokens: TokenUsage,
    /// Cost in USD.
    pub cost_usd: f64,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Whether this was served from cache.
    pub from_cache: bool,
}

/// Token usage statistics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenUsage {
    /// Input (prompt) tokens.
    pub input_tokens: i64,
    /// Output (completion) tokens.
    pub output_tokens: i64,
    /// Total tokens.
    pub total_tokens: i64,
}

/// Result from image analysis (vision).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageAnalysis {
    /// Extracted layout description.
    pub layout: Option<serde_json::Value>,
    /// Detected visual types and their positions.
    pub visuals: Vec<DetectedVisual>,
    /// Extracted color palette.
    pub color_palette: Vec<String>,
    /// Detected text/labels in the image.
    pub text_labels: Vec<String>,
    /// Overall style description.
    pub style_description: String,
}

/// A visual detected in an image.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DetectedVisual {
    /// Visual type.
    pub visual_type: String,
    /// Position description.
    pub position: String,
    /// Approximate bounding box.
    pub bounds: Option<BoundingBox>,
    /// Associated text labels.
    pub labels: Vec<String>,
}

/// A bounding box for a detected visual.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BoundingBox {
    /// Left edge (0.0 - 1.0, fraction of image width).
    pub left: f64,
    /// Top edge.
    pub top: f64,
    /// Width.
    pub width: f64,
    /// Height.
    pub height: f64,
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM
// ═══════════════════════════════════════════════════════════════════════

/// A record of a database schema migration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaMigration {
    /// Migration version number.
    pub version: i32,
    /// Migration name / description.
    pub name: String,
    /// When the migration was applied.
    pub applied_at: DateTime<Utc>,
    /// Checksum of the migration SQL.
    pub checksum: String,
}

/// System-level settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemSettings {
    /// Settings key.
    pub key: String,
    /// Settings value (JSON).
    pub value: serde_json::Value,
    /// Description.
    pub description: Option<String>,
    /// When last modified.
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_ordering() {
        assert!(UserRole::Admin.can(UserRole::Designer));
        assert!(UserRole::Designer.can(UserRole::Analyst));
        assert!(!UserRole::Viewer.can(UserRole::Admin));
    }

    #[test]
    fn test_user_role_parse() {
        assert_eq!(UserRole::parse("admin"), Some(UserRole::Admin));
        assert_eq!(UserRole::parse("SUPER_ADMIN"), Some(UserRole::SuperAdmin));
        assert_eq!(UserRole::parse("unknown"), None);
    }

    #[test]
    fn test_dax_complexity_parse() {
        assert_eq!(DaxComplexity::parse("intermediate"), Some(DaxComplexity::Intermediate));
        assert_eq!(DaxComplexity::parse("EXPERT"), Some(DaxComplexity::Expert));
        assert_eq!(DaxComplexity::parse("invalid"), None);
    }

    #[test]
    fn test_schema_metadata_serialization() {
        let schema = SchemaMetadata {
            name: "Test Model".into(),
            description: None,
            tables: vec![],
            relationships: vec![],
            measures: vec![],
            calculation_groups: vec![],
            annotations: vec![],
            discovered_at: Utc::now(),
            compatibility_level: 1600,
            default_mode: Some("Import".into()),
        };
        let json = serde_json::to_string(&schema).unwrap();
        let parsed: SchemaMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Test Model");
        assert_eq!(parsed.compatibility_level, 1600);
    }
}
