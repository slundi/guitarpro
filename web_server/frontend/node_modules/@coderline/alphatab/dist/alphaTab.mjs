/*!
* alphaTab v1.8.3 (, build 32)
*
* Copyright © 2026, Daniel Kuschny and Contributors, All rights reserved.
*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/.
*
* Integrated Libraries:
*
* Library: TinySoundFont
* License: MIT
* Copyright: Copyright (C) 2017, 2018 Bernhard Schelling
* URL: https://github.com/schellingb/TinySoundFont
* Purpose: SoundFont loading and Audio Synthesis
*
* Library: SFZero
* License: MIT
* Copyright: Copyright (C) 2012 Steve Folta ()
* URL: https://github.com/stevefolta/SFZero
* Purpose: TinySoundFont is based on SFZEro
*
* Library: Haxe Standard Library
* License: MIT
* Copyright: Copyright (C)2005-2025 Haxe Foundation
* URL: https://github.com/HaxeFoundation/haxe/tree/development/std
* Purpose: XML Parser & Zip Inflate Algorithm
*
* Library: SharpZipLib
* License: MIT
* Copyright: Copyright © 2000-2018 SharpZipLib Contributors
* URL: https://github.com/icsharpcode/SharpZipLib
* Purpose: Zip Deflate Algorithm for writing compressed Zips
*
* Library: NVorbis
* License: MIT
* Copyright: Copyright (c) 2020 Andrew Ward
* URL: https://github.com/NVorbis/NVorbis
* Purpose: Vorbis Stream Decoding
*
* Library: libvorbis
* License: BSD-3-Clause
* Copyright: Copyright (c) 2002-2020 Xiph.org Foundation
* URL: https://github.com/xiph/vorbis
* Purpose: NVorbis adopted some code from libvorbis.
*
* @preserve
* @license
*/
import * as alphaTab from "./alphaTab.core.mjs";
export * from "./alphaTab.core.mjs";
//#region src/alphaTab.main.ts
if (alphaTab.Environment.isRunningInWorker) alphaTab.Environment.initializeWorker();
else if (alphaTab.Environment.isRunningInAudioWorklet) alphaTab.Environment.initializeAudioWorklet();
else alphaTab.Environment.initializeMain((settings) => {
	if (alphaTab.Environment.webPlatform === alphaTab.WebPlatform.NodeJs) throw new alphaTab.AlphaTabError(alphaTab.AlphaTabErrorType.General, "Workers not yet supported in Node.js");
	if (alphaTab.Environment.webPlatform === alphaTab.WebPlatform.BrowserModule || alphaTab.Environment.isWebPackBundled || alphaTab.Environment.isViteBundled) {
		alphaTab.Logger.debug("AlphaTab", "Creating webworker");
		try {
			return new alphaTab.Environment.alphaTabWorker(new alphaTab.Environment.alphaTabUrl("./alphaTab.worker.mjs", import.meta.url), { type: "module" });
		} catch (e) {
			alphaTab.Logger.debug("AlphaTab", "ESM webworker construction with direct URL failed", e);
		}
		let workerUrl = "";
		try {
			workerUrl = new alphaTab.Environment.alphaTabUrl("./alphaTab.worker.mjs", import.meta.url);
			const script = `import ${JSON.stringify(workerUrl)}`;
			const blob = new Blob([script], { type: "application/javascript" });
			return new Worker(URL.createObjectURL(blob), { type: "module" });
		} catch (e) {
			alphaTab.Logger.debug("AlphaTab", "ESM webworker construction with blob import failed", workerUrl, e);
		}
		try {
			if (!settings.core.scriptFile) throw new Error("Could not detect alphaTab script file");
			workerUrl = settings.core.scriptFile;
			const script = `import ${JSON.stringify(settings.core.scriptFile)}`;
			const blob = new Blob([script], { type: "application/javascript" });
			return new Worker(URL.createObjectURL(blob), { type: "module" });
		} catch (e) {
			alphaTab.Logger.debug("AlphaTab", "ESM webworker construction with blob import failed", settings.core.scriptFile, e);
		}
	}
	if (!settings.core.scriptFile) throw new alphaTab.AlphaTabError(alphaTab.AlphaTabErrorType.General, "Could not detect alphaTab script file, cannot initialize renderer");
	try {
		alphaTab.Logger.debug("AlphaTab", "Creating Blob worker");
		const script = `importScripts('${settings.core.scriptFile}')`;
		const blob = new Blob([script], { type: "application/javascript" });
		return new Worker(URL.createObjectURL(blob));
	} catch {
		alphaTab.Logger.warning("Rendering", "Could not create inline worker, fallback to normal worker");
		return new Worker(settings.core.scriptFile);
	}
}, (context, settings) => {
	if (alphaTab.Environment.webPlatform === alphaTab.WebPlatform.NodeJs) throw new alphaTab.AlphaTabError(alphaTab.AlphaTabErrorType.General, "Audio Worklets not yet supported in Node.js");
	if (alphaTab.Environment.webPlatform === alphaTab.WebPlatform.BrowserModule || alphaTab.Environment.isWebPackBundled || alphaTab.Environment.isViteBundled) {
		alphaTab.Logger.debug("AlphaTab", "Creating Module worklet");
		const { audioWorklet: alphaTabWorklet } = context;
		return alphaTabWorklet.addModule(new alphaTab.Environment.alphaTabUrl("./alphaTab.worklet.mjs", import.meta.url));
	}
	alphaTab.Logger.debug("AlphaTab", "Creating Script worklet");
	return context.audioWorklet.addModule(settings.core.scriptFile);
});
//#endregion
