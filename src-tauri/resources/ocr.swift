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
            print("[]")
            exit(0)
        }
        
        let imageW = CGFloat(cgImage.width)
        let imageH = CGFloat(cgImage.height)
        
        var regions: [[String: Any]] = []
        for obs in observations {
            let bbox = obs.boundingBox
            if let text = obs.topCandidates(1).first?.string, !text.isEmpty {
                regions.append([
                    "text": text,
                    "x": Double(bbox.origin.x),
                    "y": Double(bbox.origin.y),
                    "w": Double(bbox.size.width),
                    "h": Double(bbox.size.height),
                ])
            }
        }
        
        let json = try JSONSerialization.data(withJSONObject: regions, options: [])
        print(String(data: json, encoding: .utf8) ?? "[]")
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
