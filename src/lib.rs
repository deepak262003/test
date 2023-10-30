
mod exports;
mod fonts;
mod world;

use base64::decode;
use exports::{export_image, export_pdf,ImageExportFormat};
use std::ffi::c_char;
use std::ffi::{CStr, CString};
use std::fs;
use std::cell::Cell;
use std::collections::HashMap;
use std::process::ExitCode;
use typst::eval::Tracer;
use typst::diag::{bail, StrResult};
use typst::doc::Document;
use world::SimpleWorld;


#[no_mangle]
pub extern "C" fn TypstCreate(
    typst_source: *const c_char,
    output_format: *const c_char,
    image_data:  *const c_char,  //image data as string (format : filename.ext-base64data ...) (seperator : whitespace)
    template_data: *const c_char //template data as string (format : filename.typ@@@template data||| ...) (seperator: |||)
) -> *mut c_char {
    let typst_source_string = get_string(typst_source);
    let output_format_string = get_string(output_format);
    let image_data_string = get_string(image_data);
    let templates_data_string = get_string(template_data);

    let mut image_name_hash:HashMap<String,usize> = HashMap::new(); //maps file name to position of bytes data in vector(images_data)
    let mut templates_data_hash:HashMap<String,usize> = HashMap::new(); //maps file name to data string in vector(templates_data)

    let mut images_data: Vec<Vec<u8>> = Vec::new(); //vector of image bytes
    let mut images_name_data: Vec<String> = Vec::new(); //vector of whitespace splitted image_data


    if image_data_string!="empty"&& image_data_string.len()>0{  //prevents execution when image_data_string is empty
    images_name_data = image_data_string.split_whitespace().map(|s| s.to_string()).collect();

    //gets image data and position and assigns it to hashmap
    // maps position to the corresponding file name
    let mut decoded_bytes = Vec::new();
    for (index,image_data) in images_name_data.iter().enumerate() {
        println!("{}",image_data);
        let image_data_split:Vec<&str> = image_data.split('-').collect(); 
        image_name_hash.insert(image_data_split.get(0).unwrap().to_string(),index);
        decoded_bytes = decode(image_data_split.get(1).unwrap()).unwrap();
        images_data.push(decoded_bytes);
    }
    }

    let mut templates_content: Vec<String> = Vec::new(); //vector of template string data
    let mut templates_name_data: Vec<String> = Vec::new(); // vector of ||| splitted templates_data

    if templates_data_string!="empty"&& templates_data_string.len()>0{ //prevents execution when templates_data_string is empty
        templates_name_data = templates_data_string.split("|||").map(|s| s.to_string()).collect();
    
    //gets template string data and position and assigns it to hashmap
    //maps position to corresponding file name
    for (index,templates_data) in templates_name_data.iter().enumerate() {
        println!("{}",templates_data);
        let templates_data_split:Vec<&str> = templates_data.split("@@@").collect();
        templates_data_hash.insert(templates_data_split.get(0).unwrap().to_string(),index);
        templates_content.push(templates_data_split.get(1).unwrap().to_string()); 
    }
    }
   
    let world = SimpleWorld::new(typst_source_string,images_data,image_name_hash,templates_content,templates_data_hash).unwrap();
    let mut tracer = Tracer::default();

    let result = typst::compile(&world, &mut tracer);

    match result {
        Ok(document) => {
            let base64_val = export(&document, output_format_string).unwrap();
            println!("{:#?}",base64_val);

            println!("Compilation succeeded");
            
            
            let cstring = CString::new(base64_val).expect("CString Failed");

            cstring.into_raw()
        }

        Err(_error) => {
            let error_message = format!("Compilation Failed: {:?}",_error);
            println!("{}",error_message);
            let c_string = CString::new(error_message).expect("Failed to create CString");
            c_string.into_raw()
        }
    }
}

thread_local! {
    /// The CLI's exit code.
    static EXIT: Cell<ExitCode> = Cell::new(ExitCode::SUCCESS);
}

fn get_string(s: *const c_char) -> String {
    let c_str = unsafe {
        assert!(!s.is_null());

        CStr::from_ptr(s)
    };

    let r_str = c_str
        .to_str()
        .expect("Could not successfully convert string form foreign code!");

    String::from(r_str)
}

/// Ensure a failure exit code.
fn set_failed() {
    EXIT.with(|cell| cell.set(ExitCode::FAILURE));
}

/// Export into the target format.
fn export(document: &Document, output_format: String) -> StrResult<String> {
    match output_format.as_str() {
        "pdf" => return export_pdf(&document),
        "png" => return export_image(&document, ImageExportFormat::Png, 144.0),
        "svg" => return export_image(&document, ImageExportFormat::Svg, 144.0),
        _ => bail!("Couldn't understand the format!"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose, Engine as _};

    #[test]
    fn c_string_test_pdf() {
        env_logger::init();
        let text = r#"#import "conf.typ": conf
        #show: doc => conf(
          title: [
            Towards Improved Modelling
          ],
          authors: (
            (
              name: "Theresa Tungsten",
              affiliation: "Artos Institute",
              email: "tung@artos.edu",
            ),
            (
              name: "Eugene Deklan",
              affiliation: "Honduras State",
              email: "e.deklan@hstate.hn",
            ),
          ),
          abstract: lorem(80),
          doc,
        )
        
       "#;
        let file_name = r#"empty"#;  //images data (format : filename1.ext-base64data filename2.ext-base64data )
        let templates= r#""#; //templates data (format : conf1.typ@@@template content|||conf2.typ@@@template content)
        let text_bytes = text.as_bytes();

        let text_encoded: String = general_purpose::STANDARD.encode(text_bytes);
        let text_encoded_raw_cstr = CString::new(text_encoded).unwrap();
        let output_format_raw_cstr = CString::new("pdf".to_string()).unwrap();
        let templates_raw_cstr=CString::new(templates.to_string()).unwrap();
        let file_name_raw_str= CString::new(file_name.to_string()).unwrap();
        let text_encoded_ptr = text_encoded_raw_cstr.as_ptr();
        let output_format_ptr = output_format_raw_cstr.as_ptr();
        let file_name_ptr= file_name_raw_str.as_ptr();
        let templates_ptr = templates_raw_cstr.as_ptr();
          
        let _base64_pdf = TypstCreate(text_encoded_ptr, output_format_ptr,file_name_ptr,templates_ptr);
  
        println!("{:#?}", _base64_pdf);
      
    }

    // #[test]
    // fn c_string_test_png() {
    //     let text = r#"#let amazed(term, color: blue) = {
	// 						  text(color, box[✨ #term ✨])
	// 							}
	// 						You are #amazed[beautiful]!
	// 						I am #amazed(color: purple)[amazed]!"#;

    //         let file_name = r#"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"#;
    //         let text_bytes = text.as_bytes();
    //         let text_encoded: String = general_purpose::STANDARD.encode(text_bytes);
    //         let text_encoded_raw_cstr = CString::new(text_encoded).unwrap();
    //         let output_format_raw_cstr = CString::new("png".to_string()).unwrap();
    //         let file_name_raw_str= CString::new(file_name.to_string()).unwrap();
    //         let text_encoded_ptr = text_encoded_raw_cstr.as_ptr();
    //         let output_format_ptr = output_format_raw_cstr.as_ptr();
    //         let file_name_ptr= file_name_raw_str.as_ptr();
                
    //         let _base64_png = TypstCreate(text_encoded_ptr, output_format_ptr,file_name_ptr);

    //     println!("{:#?} _base64_png", _base64_png);
    // }
    

    // #[test]
    // fn c_string_test_svg() {
    //     let text = r#"#let amazed(term, color: blue) = {
	// 						  text(color, box[✨ #term ✨])
	// 							}
	// 						You are #amazed[beautiful]!
    //         I am #amazed(color: purple)[amazed]!"#;
    //         let file_name = r#"house1.jpeg house2.jpeg"#;
    //         let text_bytes = text.as_bytes();
    //         let text_encoded: String = general_purpose::STANDARD.encode(text_bytes);
    //         let text_encoded_raw_cstr = CString::new(text_encoded).unwrap();
    //         let output_format_raw_cstr = CString::new("svg".to_string()).unwrap();
    //         let file_name_raw_str= CString::new(file_name.to_string()).unwrap();
    //         let text_encoded_ptr = text_encoded_raw_cstr.as_ptr();
    //         let output_format_ptr = output_format_raw_cstr.as_ptr();
    //         let file_name_ptr= file_name_raw_str.as_ptr();
                
    //         let _base64_svg = TypstCreate(text_encoded_ptr, output_format_ptr,file_name_ptr);
                      
    //     println!("{:#?} _base64_svg ", _base64_svg);
    // }

}
