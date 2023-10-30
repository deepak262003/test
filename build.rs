fn main() {
    csbindgen::Builder::default()
        .input_extern_file("src/lib.rs")
        .csharp_dll_name("typst_lib")
        .generate_csharp_file("./dotnet/TypstLib.g.cs")
        .unwrap();
}