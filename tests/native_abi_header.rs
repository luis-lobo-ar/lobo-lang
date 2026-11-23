use std::process::Command;
#[test]
fn published_native_header_rejects_incompatible_symbol_definitions() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let header = root.join("src/native_api.h");
    assert!(header.is_file(), "the ABI needs a standalone C header");
    let directory = std::env::temp_dir().join(format!("lobo-abi-header-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    for (index,source,valid) in [
      (0,"#include \"native_api.h\"\nint64_t lobo_native_roundtrip_i64_v1(int64_t value) {return value;}\n",true),
      (1,"#include \"native_api.h\"\nint32_t lobo_native_roundtrip_i64_v1(int32_t value) {return value;}\n",false),
      (2,"#include \"native_api.h\"\nlobo_native_result_v1 lobo_native_read_v1(uint64_t a,int64_t b) {(void)a;(void)b;return (lobo_native_result_v1){0};}\n",false),
    ] {
      let file=directory.join(format!("{index}.c"));std::fs::write(&file,source).unwrap();
      let output=Command::new("clang").args(["-std=c11","-Wall","-Wextra","-Werror","-fsyntax-only","-I"]).arg(root.join("src")).arg(file).output().unwrap();
      assert_eq!(output.status.success(),valid,"{output:?}");
      if !valid {assert!(String::from_utf8_lossy(&output.stderr).contains("conflicting types"));}
    }
    std::fs::remove_dir_all(directory).unwrap();
}
