use crate::{
    scan::{PluginInfo, PluginLibrary},
    util::{AUv2Id, Architecture, OperatingSystem, PluginFormat},
};
use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct BundleOptions<'a> {
    pub dylib_path: &'a Path,
    pub output_dir: &'a Path,

    pub library: &'a PluginLibrary,
    pub format: PluginFormat,

    pub os: OperatingSystem,
    pub arch: Architecture,

    pub auv2_override_id: Option<AUv2Id>,
    pub vst3_single_file: bool,
    pub overwrite_existing: bool,
}

impl BundleOptions<'_> {
    /// Create a plugin bundle in the output directory according to the specified options,
    /// and return the path to the created bundle
    pub fn bundle(self) -> Result<PathBuf> {
        if self.format == PluginFormat::Auv2 && !self.library.has_auv2_entry {
            anyhow::bail!("Plugin library does not contain an AUv2 entry point");
        }

        if self.format == PluginFormat::Vst3 && !self.library.has_vst3_entry {
            anyhow::bail!("Plugin library does not contain a VST3 entry point");
        }

        if self.format == PluginFormat::Auv2
            && self.library.plugins.len() > 1
            && self.auv2_override_id.is_some()
        {
            anyhow::bail!("AUv2 ID override only works for single-plugin libraries");
        }

        let plugin = self
            .library
            .plugins
            .first()
            .ok_or(anyhow::anyhow!("Plugin library contains no plugins"))?;

        let extension = match self.format {
            PluginFormat::Clap => "clap",
            PluginFormat::Vst3 => "vst3",
            PluginFormat::Auv2 => "component",
        };

        let output_path = self
            .output_dir
            .join(&plugin.clap_name)
            .with_extension(extension);

        if output_path.exists() {
            if self.overwrite_existing {
                if output_path.is_dir() {
                    fs::remove_dir_all(&output_path)?;
                } else {
                    fs::remove_file(&output_path)?;
                }
            } else {
                anyhow::bail!("Output path {} already exists", output_path.display());
            }
        }

        match (self.format, self.os) {
            (_, OperatingSystem::MacOS) => {
                let info_plist = if self.format == PluginFormat::Auv2 {
                    info_plist_auv2(plugin, self.library, self.auv2_override_id)?
                } else {
                    info_plist_generic(plugin)
                };

                fs::create_dir_all(&output_path)?;
                fs::create_dir_all(output_path.join("Contents/MacOS"))?;
                fs::write(output_path.join("Contents/PkgInfo"), "BNDL????")?;
                fs::write(output_path.join("Contents/Info.plist"), info_plist)?;
                reflink::reflink_or_copy(
                    self.dylib_path,
                    output_path.join("Contents/MacOS").join(
                        output_path
                            .with_extension("")
                            .file_name()
                            .ok_or(anyhow::anyhow!("Invalid output path"))?,
                    ),
                )?;
            }

            (PluginFormat::Auv2, _) => {
                anyhow::bail!("AUv2 is only supported on macOS");
            }

            (PluginFormat::Clap, _) => {
                reflink::reflink_or_copy(self.dylib_path, &output_path)?;
            }

            (PluginFormat::Vst3, OperatingSystem::Windows) if self.vst3_single_file => {
                reflink::reflink_or_copy(self.dylib_path, &output_path)?;
            }

            (PluginFormat::Vst3, _) => {
                let os_ext = match self.os {
                    OperatingSystem::Windows => "vst3", // huh?
                    OperatingSystem::Linux => "so",
                    OperatingSystem::MacOS => "",
                };

                let os_tag = match (self.os, self.arch) {
                    (OperatingSystem::Windows, Architecture::X86) => "x86-win",
                    (OperatingSystem::Windows, Architecture::X86_64) => "x86_64-win",
                    (OperatingSystem::Windows, Architecture::Arm) => "arm-win",
                    (OperatingSystem::Windows, Architecture::Aarch64) => "arm64-win",
                    (OperatingSystem::Linux, Architecture::X86) => "i386-linux",
                    (OperatingSystem::Linux, Architecture::X86_64) => "x86_64-linux",
                    (OperatingSystem::Linux, Architecture::Arm) => todo!(),
                    (OperatingSystem::Linux, Architecture::Aarch64) => todo!(),
                    (OperatingSystem::MacOS, _) => "MacOS",
                };

                fs::create_dir_all(&output_path)?;
                fs::create_dir_all(output_path.join("Contents").join(os_tag))?;
                reflink::reflink_or_copy(
                    self.dylib_path,
                    output_path
                        .join("Contents")
                        .join(os_tag)
                        .join(&plugin.clap_name)
                        .with_extension(os_ext),
                )?;
            }
        }

        Ok(output_path)
    }
}

/// Generate a generic MacOS bundle Info.plist file for the given plugin.
fn info_plist_generic(plugin: &PluginInfo) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist>
  <dict>
    <key>CFBundleExecutable</key>
    <string>{}</string>
    <key>CFBundleIconFile</key>
    <string></string>
    <key>CFBundleIdentifier</key>
    <string>{}</string>
    <key>CFBundleName</key>
    <string>{}</string>
    <key>CFBundleDisplayName</key>
    <string>{}</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleShortVersionString</key>
    <string>{}</string>
    <key>CFBundleVersion</key>
    <string>{}</string>
    <key>CFBundleSupportedPlatforms</key>
	<array>
	  <string>MacOSX</string>
	</array>
    <key>NSHumanReadableCopyright</key>
    <string>{}</string>
    <key>NSHighResolutionCapable</key>
    <true/>
  </dict>
</plist>"#,
        plugin.clap_name,
        plugin.clap_id,
        plugin.clap_name,
        plugin.clap_name,
        plugin.clap_version.as_deref().unwrap_or("0.0.0"),
        plugin.clap_version.as_deref().unwrap_or("0.0.0"),
        plugin.clap_vendor.as_deref().unwrap_or(""),
    )
}

/// Generate a MacOS bundle Info.plist file for the given plugin, including AUv2-specific metadata if available
fn info_plist_auv2(
    plugin: &PluginInfo,
    library: &PluginLibrary,
    auv2_override_id: Option<AUv2Id>,
) -> Result<String> {
    let mut buffer = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist>
  <dict>
    <key>CFBundleExecutable</key>
    <string>{}</string>
    <key>CFBundleIconFile</key>
    <string></string>
    <key>CFBundleIdentifier</key>
    <string>{}</string>
    <key>CFBundleName</key>
    <string>{}</string>
    <key>CFBundleDisplayName</key>
    <string>{}</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleShortVersionString</key>
    <string>{}</string>
    <key>CFBundleVersion</key>
    <string>{}</string>
    <key>CFBundleSupportedPlatforms</key>
    <array>
      <string>MacOSX</string>
    </array>
    <key>NSHumanReadableCopyright</key>
    <string>{}</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSPrincipalClass</key>
    <string/>
    <key>AudioComponents</key>
    <array>"#,
        plugin.clap_name,
        plugin.clap_id,
        plugin.clap_name,
        plugin.clap_name,
        plugin.clap_version.as_deref().unwrap_or("0.0.0"),
        plugin.clap_version.as_deref().unwrap_or("0.0.0"),
        plugin.clap_vendor.as_deref().unwrap_or(""),
    );

    // we currently support at most 4 plugins per bundle
    for (index, plugin) in library.plugins.iter().take(4).enumerate() {
        let auv2_id = auv2_override_id.or(plugin.auv2_id).ok_or_else(|| {
            anyhow::anyhow!("Plugin does not export an AUv2 ID and no override was provided")
        })?;

        let code_manu = str::from_utf8(&auv2_id.manufacturer)
            .map_err(|_| anyhow::anyhow!("Non-ASCII AUv2 manufacturer code"))?;
        let code_subt = str::from_utf8(&auv2_id.subtype)
            .map_err(|_| anyhow::anyhow!("Non-ASCII AUv2 subtype code"))?;
        let code_type = str::from_utf8(&auv2_id.type_)
            .map_err(|_| anyhow::anyhow!("Non-ASCII AUv2 type code"))?;

        let version = parse_version_auv2(plugin.clap_version.as_deref().unwrap_or("0.0.0"));

        buffer.push_str(&format!(
            r#"
      <dict>
        <key>name</key>
        <string>{}</string>
        <key>description</key>
        <string>{}</string>
        <key>factoryFunction</key>
        <string>GetPluginFactoryAUV2_{}</string>
        <key>manufacturer</key>
        <string>{}</string>
        <key>subtype</key>
        <string>{}</string>
        <key>type</key>
        <string>{}</string>
        <key>version</key>
        <integer>{}</integer>
        <key>resourceUsage</key>
        <dict>
           <key>network.client</key>
           <true/>
           <key>temporary-exception.files.all.read-write</key>
           <true/>
        </dict>
      </dict>"#,
            plugin.clap_name,
            plugin.clap_description.as_deref().unwrap_or(""),
            index,
            code_manu,
            code_subt,
            code_type,
            version
        ));
    }

    buffer.push_str(
        r#"
    </array>
  </dict>
</plist>"#,
    );

    Ok(buffer)
}

/// A very lenient version parser that just extracts the first three dot-separated numbers from the version string, ignoring any non-numeric prefix or suffix
///
/// Used for generating an Info.plist file for AUv2 plugins
fn parse_version_auv2(version: &str) -> u32 {
    let mut parts = version
        .strip_prefix(char::is_alphabetic)
        .unwrap_or(version)
        .split('.')
        .map(|s| s.parse::<u8>().unwrap_or(0));
    let major = parts.next().unwrap_or(0);
    let minor = parts.next().unwrap_or(0);
    let patch = parts.next().unwrap_or(0);

    (major as u32) << 16 | (minor as u32) << 8 | patch as u32
}
