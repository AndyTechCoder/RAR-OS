#![deny(unsafe_code)]

#[allow(dead_code)]
#[path = "../../../tools/rar-lab/safety/src/lib.rs"]
mod safety;

use safety::{CertificationRecord, CommandPlan, VmProfile, sha256_hex};

fn main() {
    let profile_text = include_str!(
        "../../../spec/lab/vm-profile/examples/x86_64-preauth.profile"
    );
    let command_text = include_str!(
        "../../../spec/lab/vm-profile/examples/x86_64-preauth.command"
    );
    let closure_text = include_str!(
        "../../../spec/lab/preauth/locks/r0-x86_64-preauth-v2.lock"
    );
    let packages_text = include_str!(
        "../../../spec/lab/preauth/locks/r0-x86_64-preauth-packages.v2"
    );
    let disk_text = include_str!(
        "../../../spec/lab/preauth/locks/r0-x86_64-preauth-disk.v1"
    );
    let certification_text = include_str!(
        "../../../spec/lab/vm-profile/prepared/r0-x86_64-preauth-v1.cert"
    );

    let profile = VmProfile::parse(profile_text).expect("canonical preauth profile");
    assert_eq!(profile.sha256(), "8e7bc38fa513700556b7ea493ffd42b6df6b4adcaf0a4719a0c7fe11f7eb165f");
    let command = CommandPlan::from_profile(&profile);
    assert_eq!(command.canonical(), command_text);
    assert_eq!(command.sha256(), "7d8e5f500c35b5da4de0d3f2a6d9b667563bb6e3ff7ed6503192ee8e69e0550d");
    assert_eq!(sha256_hex(closure_text.as_bytes()), "b4ad190e4254fdc2a8e60e77b49781ad4ed94a5746c1a58ae342629631215a21");
    assert!(closure_text.contains("derived_oci_index_sha256=4d59826fb248130555b99aa6bc034f17db7df4a6acbbe1ebc0a8175492476531\n"));
    assert!(closure_text.contains("certifiable=true\n"));
    assert_eq!(sha256_hex(packages_text.as_bytes()), "5f73693e6202969af6ca958e28bb4ce189741f4130f40e2eb5bb593e2e07519a");
    assert_eq!(packages_text.lines().filter(|line| line.starts_with("package|")).count(), 36);
    assert!(disk_text.contains("seed_sha256=141d4f9b5756451e4d5874ac2d68c5c59052b82e52494d29ef8624fa3402e766\n"));
    assert!(disk_text.contains("child_sha256=141d4f9b5756451e4d5874ac2d68c5c59052b82e52494d29ef8624fa3402e766\n"));
    assert!(disk_text.ends_with("record_sha256=89e160c117154dded20d7daeaf75576dd082a9d83d5ade47f9254a6e35371826\n"));

    let certification = CertificationRecord::parse(certification_text)
        .expect("integrity-bound prepared certification");
    assert_eq!(certification.profile_sha256, profile.sha256());
    assert_eq!(certification.command_sha256, command.sha256());
    assert_eq!(certification.tool_lock_sha256, sha256_hex(closure_text.as_bytes()));
    assert_eq!(certification.reviewer, "pending-independent-review");
}
