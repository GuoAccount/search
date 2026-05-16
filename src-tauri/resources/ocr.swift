#!/usr/bin/env swift

import Foundation
import Vision
import AppKit

func performOCR(on imagePath: String) {
    let url = URL(fileURLWithPath: imagePath)
    
    guard let image = NSImage(contentsOf: url) else {
        print("ERROR: Cannot load image")
        exit(1)
    }
    
    guard let cgImage = image.cgImage(forProposedRect: nil, context: nil, hints: nil) else {
        print("ERROR: Cannot convert to CGImage")
        exit(1)
    }
    
    let request = VNRecognizeTextRequest()
    request.recognitionLevel = .accurate
    request.recognitionLanguages = ["zh-Hans", "zh-Hant", "en-US"]
    request.usesLanguageCorrection = true
    
    let handler = VNImageRequestHandler(cgImage: cgImage, options: [:])
    
    do {
        try handler.perform([request])
        
        guard let observations = request.results else {
            print("")
            exit(0)
        }
        
        let recognizedText = observations
            .compactMap { $0.topCandidates(1).first?.string }
            .joined(separator: "\n")
        
        print(recognizedText)
    } catch {
        print("ERROR: \(error.localizedDescription)")
        exit(1)
    }
}

// Main
if CommandLine.arguments.count < 2 {
    print("ERROR: Usage: ocr.swift <image_path>")
    exit(1)
}

let imagePath = CommandLine.arguments[1]
performOCR(on: imagePath)
