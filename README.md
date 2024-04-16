## How the workflow works

This crate is to be used on both the client and the server, I will explain it using a single function

```rust
fn main() {

    const PATH_AIRCRAFT_OLD_REPO:&str = "C:/C17V1.2"
    const PATH_AIRCRAFT_NEW_REPO:&str = "C:/C17V1.3"

    // the first operation that needs to happen is on the build side, when a project is built you must call:
    infinity_download_fragmenter::hash_aircraft(PATH_AIRCRAFT_NEW_REPO, "1.3", output_path).expect("failed to hash aircraft");
    // this function will generate a hash.json which will be used to detect changes and create patches, it should be included with every build

    // now the server can look at both outputs and compare then using the following:
    infinity_download_fragmenter::compare_hash(path_to_old_hash, path_to_new_hash, output_path).expect("failed to compare hash");
    // this function generates a map.json which will be used by the diff functions, this needs to be hosted by a REST API once created

    // now its time to diff files
    infinity_download_fragmenter::dif_from_map("map.json from before", PATH_AIRCRAFT_OLD_REPO, PATH_AIRCRAFT_NEW_REPO, output_path).expect("Failed to diff aircraft");
    // this function will generate a .download file containing all the binary instruction on updating every file, this gets hosted by CDN


    // now on the client side: we need to determine aircraft version (look at map.version) and get the appropriate .download from the cdn
    // next patch the aircraft, parse .download and use that in patch function
    let patch:HashMap<String, Vec<u8>> = infinity_download_fragmenter::parse_patch_file(path).expect("Failed to Parse Patch");
    infinity_download_fragmenter::patch_via_map(map.json_from_api, patch, PATH_AIRCRAFT_OLD_REPO).expect("Failed to Patch");
}
```

and there it is; no unzipping, very fast patches, and small downloads
