import re

def patch(path, old, new):
    s = open(path).read()
    n = s.count(old)
    if n:
        s = s.replace(old, new)
        open(path, 'w').write(s)
    return n

# ---- connections.rs: per-item dead-code attributes ----
base = 'crates/rdp-client/src/'
c = 0
c += patch(base + 'connections.rs',
           'pub struct ConnectionStore {',
           '#[cfg_attr(not(windows), allow(dead_code))] // persisted-connection store used by the Windows saved-connection UI\npub struct ConnectionStore {')
c += patch(base + 'connections.rs',
           'impl ConnectionStore {',
           '#[cfg_attr(not(windows), allow(dead_code))] // store methods used by the Windows saved-connection UI\nimpl ConnectionStore {')
c += patch(base + 'connections.rs',
           'fn default_store_path() -> PathBuf {',
           '#[cfg_attr(not(windows), allow(dead_code))] // store persistence path used by the Windows saved-connection UI\nfn default_store_path() -> PathBuf {')
c += patch(base + 'connections.rs',
           'fn corrupt_path(path: &Path) -> PathBuf {',
           '#[cfg_attr(not(windows), allow(dead_code))] // store recovery helper used by the Windows saved-connection UI\nfn corrupt_path(path: &Path) -> PathBuf {')

# ---- main.rs: QualityPreset derive ----
old_qp = """#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QualityPreset {
    /// Motion-first: render-scale friendly, upscaler tuned for game imagery.
    Gaming,
    /// Clarity-first: no render-scale, smooth vsync, bicubic.
    Office,
    /// The defaults (identical codec caps; see the enum docs).
    Balanced,
}

impl Default for QualityPreset {
    fn default() -> Self {
        QualityPreset::Balanced
    }
}"""
new_qp = """#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum QualityPreset {
    /// Motion-first: render-scale friendly, upscaler tuned for game imagery.
    Gaming,
    /// Clarity-first: no render-scale, smooth vsync, bicubic.
    Office,
    /// The defaults (identical codec caps; see the enum docs).
    #[default]
    Balanced,
}"""
q = patch(base + 'main.rs', old_qp, new_qp)

# ---- main.rs: MonitorLayout type alias (type_complexity) ----
old_ml = """#[allow(dead_code)] // used by the Windows multi-monitor layout path and unit tests
fn scale_monitor_layout(
    rects: &[rdp_pdu::gcc::VirtualScreenRect],
    scale: f32,
) -> (
    Vec<rdp_pdu::gcc::MonitorDef>,
    (u32, u32),
    Vec<((u32, u32), (u32, u32))>,
) {"""
new_ml = """/// Scaled multi-monitor layout: the monitor defs to advertise, the scaled
/// desktop size, and each monitor's framebuffer slice ((origin), (size)).
#[cfg_attr(not(windows), allow(dead_code))] // used by the Windows multi-monitor layout path
type MonitorLayout = (
    Vec<rdp_pdu::gcc::MonitorDef>,
    (u32, u32),
    Vec<((u32, u32), (u32, u32))>,
);

#[allow(dead_code)] // used by the Windows multi-monitor layout path and unit tests
fn scale_monitor_layout(rects: &[rdp_pdu::gcc::VirtualScreenRect], scale: f32) -> MonitorLayout {"""
m = patch(base + 'main.rs', old_ml, new_ml)

# ---- w365.rs: field_reassign_with_default ----
old_w = """        let mut entry = crate::feed::FeedEntry::default();
        entry.display_name = settings
            .get("remotedesktopname")
            .filter(|s| !s.is_empty())
            .cloned()
            .unwrap_or_else(|| "Cloud PC".to_string());
        // `remoteapplicationprogram` is `||<resourceId>`; the GUID distinguishes
        // Cloud PCs that share a SKU display name. Used only as a picker label.
        entry.resource_id = settings
            .get("remoteapplicationprogram")
            .map(|s| s.trim_start_matches('|').to_string())
            .unwrap_or_default();
        entry.tenant_id = settings.get("aadtenantid").cloned().unwrap_or_default();
        entry.gateway_fqdn = settings.get("gatewayhostname").cloned().unwrap_or_default();
        entry.load_balance_info = Some(lbi.into_bytes());
        entry.rdp_file = Some(rdp_contents);
        entries.push(entry);"""
new_w = """        let entry = crate::feed::FeedEntry {
            display_name: settings
                .get("remotedesktopname")
                .filter(|s| !s.is_empty())
                .cloned()
                .unwrap_or_else(|| "Cloud PC".to_string()),
            // `remoteapplicationprogram` is `||<resourceId>`; the GUID
            // distinguishes Cloud PCs that share a SKU display name. Used only
            // as a picker label.
            resource_id: settings
                .get("remoteapplicationprogram")
                .map(|s| s.trim_start_matches('|').to_string())
                .unwrap_or_default(),
            tenant_id: settings.get("aadtenantid").cloned().unwrap_or_default(),
            gateway_fqdn: settings.get("gatewayhostname").cloned().unwrap_or_default(),
            load_balance_info: Some(lbi.into_bytes()),
            rdp_file: Some(rdp_contents),
            ..crate::feed::FeedEntry::default()
        };
        entries.push(entry);"""
w = patch(base + 'w365.rs', old_w, new_w)

# ---- stun.rs: io_other_error (5 occurrences) ----
sp = base + 'stun.rs'
s = open(sp).read()
s2 = re.sub(r'io::Error::new\(\s*ErrorKind::Other,\s*("[^"]*")\s*\)', r'io::Error::other(\1)', s)
st = len(re.findall(r'io::Error::new\(\s*ErrorKind::Other,', s))
open(sp, 'w').write(s2)

print('A' * c)
print('B' * q)
print('C' * m)
print('D' * w)
print('E' * st)
