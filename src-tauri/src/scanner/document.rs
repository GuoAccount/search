use std::fs;
use std::io::Read;
use std::path::Path;

pub fn extract_docx_text(path: &Path) -> Result<String, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut doc = archive.by_name("word/document.xml").map_err(|e| e.to_string())?;
    let mut contents = String::new();
    doc.read_to_string(&mut contents).map_err(|e| e.to_string())?;
    let mut text = String::new();
    let mut reader = quick_xml::Reader::from_str(&contents);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(ref e)) | Ok(quick_xml::events::Event::Empty(ref e)) => {
                if e.local_name().as_ref() == b"t" {
                    if let Ok(quick_xml::events::Event::Text(t)) = reader.read_event_into(&mut Vec::new()) {
                        let s = t.unescape().unwrap_or_default();
                        if !s.is_empty() {
                            text.push_str(&s);
                            text.push('\n');
                        }
                    }
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(text)
}

pub fn extract_xlsx_text(path: &Path) -> Result<String, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut shared_strings = Vec::new();
    if let Ok(mut ss_file) = archive.by_name("xl/sharedStrings.xml") {
        let mut contents = String::new();
        ss_file.read_to_string(&mut contents).map_err(|e| e.to_string())?;
        let mut reader = quick_xml::Reader::from_str(&contents);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(ref e)) | Ok(quick_xml::events::Event::Empty(ref e)) => {
                    if e.local_name().as_ref() == b"t" {
                        if let Ok(quick_xml::events::Event::Text(t)) = reader.read_event_into(&mut Vec::new()) {
                            shared_strings.push(t.unescape().unwrap_or_default().to_string());
                        }
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }
    }
    let mut sheet_names: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        if let Ok(name) = archive.by_index(i) {
            let n = name.name().to_string();
            if n.starts_with("xl/worksheets/sheet") && n.ends_with(".xml") {
                sheet_names.push(n);
            }
        }
    }
    let mut text = String::new();
    for sheet_name in sheet_names {
        if let Ok(mut sheet_file) = archive.by_name(&sheet_name) {
            let mut contents = String::new();
            sheet_file.read_to_string(&mut contents).map_err(|e| e.to_string())?;
            let mut reader = quick_xml::Reader::from_str(&contents);
            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(quick_xml::events::Event::Start(ref e)) | Ok(quick_xml::events::Event::Empty(ref e)) => {
                        if e.local_name().as_ref() == b"v" {
                            if let Ok(quick_xml::events::Event::Text(t)) = reader.read_event_into(&mut Vec::new()) {
                                let val = t.unescape().unwrap_or_default().to_string();
                                if let Ok(idx) = val.parse::<usize>() {
                                    if let Some(s) = shared_strings.get(idx) {
                                        text.push_str(s);
                                        text.push('\n');
                                    }
                                } else {
                                    text.push_str(&val);
                                    text.push('\n');
                                }
                            }
                        }
                    }
                    Ok(quick_xml::events::Event::Eof) => break,
                    Err(_) => break,
                    _ => {}
                }
                buf.clear();
            }
        }
    }
    Ok(text)
}

pub fn extract_pptx_text(path: &Path) -> Result<String, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut slide_names: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let n = entry.name().to_string();
            if n.starts_with("ppt/slides/slide") && n.ends_with(".xml") {
                slide_names.push(n);
            }
        }
    }
    let mut text = String::new();
    for slide_name in slide_names {
        if let Ok(mut slide_file) = archive.by_name(&slide_name) {
            let mut contents = String::new();
            slide_file.read_to_string(&mut contents).map_err(|e| e.to_string())?;
            let mut reader = quick_xml::Reader::from_str(&contents);
            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(quick_xml::events::Event::Start(ref e)) | Ok(quick_xml::events::Event::Empty(ref e)) => {
                        if e.local_name().as_ref() == b"t" {
                            if let Ok(quick_xml::events::Event::Text(t)) = reader.read_event_into(&mut Vec::new()) {
                                let s = t.unescape().unwrap_or_default();
                                if !s.is_empty() {
                                    text.push_str(&s);
                                    text.push('\n');
                                }
                            }
                        }
                    }
                    Ok(quick_xml::events::Event::Eof) => break,
                    Err(_) => break,
                    _ => {}
                }
                buf.clear();
            }
        }
    }
    Ok(text)
}

pub fn extract_pdf_text(path: &Path) -> Result<String, String> {
    let path = path.to_path_buf();
    let display = path.display().to_string();
    std::panic::catch_unwind(move || {
        pdf_extract::extract_text(&path)
    })
    .map_err(|e| {
        log::error!("PDF extraction panicked for {}: {:?}", display, e);
        "PDF extraction panicked (unsupported encoding)".to_string()
    })?
    .map_err(|e| {
        log::warn!("PDF extraction failed for {}: {}", display, e);
        e.to_string()
    })
}

pub fn extract_document_text(path: &Path, extension: &str) -> Result<String, String> {
    match extension {
        "docx" => extract_docx_text(path),
        "xlsx" => extract_xlsx_text(path),
        "pptx" => extract_pptx_text(path),
        "pdf" => extract_pdf_text(path),
        _ => Err("Unsupported document type".to_string()),
    }
}
