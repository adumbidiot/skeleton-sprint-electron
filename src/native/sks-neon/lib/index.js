var addon = require('../native');

console.log(addon.hello());

let data = ['b0', 'a1'];
console.log(addon.export1DPatch(data));

module.exports.hello = addon.hello;

module.exports.encodeBlockLBL = addon.encodeBlockLBL;

module.exports.decode = addon.decode;

module.exports.LevelBuilder = class LevelBuilder {
	constructor() {
		this.internal = new addon.LevelBuilder();
		this.internalDirty = true; //Temp until i can render from js
		this.internalData = null;
		
		this.grid = true;
		this.dirty = true;
		
	}

	enableGrid() {
		this.grid = true;
		this.dirty = true;
	}

	disableGrid() {
		this.grid = false;
		this.dirty = true;
	}

	isDirty() {
		return this.dirty;
	}

	getImage() {
		this.internal.getImage();
	}
	
	getLevelData() {
		return this.internal.getLevelData();
	}

	// Canvas MUST be 1920 x 1080
	drawImage(ctx) {
		ctx.clearRect(0, 0, 1920, 1080);
		
		if(this.internalDirty){
			let binary = new Uint8ClampedArray(this.internal.getImage());
			let imageData = new ImageData(binary, 1920, 1080);
			ctx.putImageData(imageData, 0, 0);
			this.internalData = imageData;
			this.internalDirty = false;
		}else{
			ctx.putImageData(this.internalData, 0, 0);
		}

		this.dirty = false;
	}
	
	render(ctx) {
		let boxSize = 1920 / 32;
		let data = this.internal.getLevelData();
		for(var i = 0; i < 18 * 32; i++){
			let y = (i / 32) | 0;
			let x = i % 32;
			switch(data[i]) {
				case "null": {
						break;
				}
				default: {
					//console.log(data[i]);
					let block = data[i];
					
					if(data[i].startsWith('Note:')){
						block = 'note';
					}
					
					let img = new Image();
					img.src = './images/' + block + '.png';
					
					ctx.drawImage(img, x * boxSize, y * boxSize, boxSize, boxSize);
				}
			}
			
		}
	}

	drawGrid(ctx) {
		if(!this.grid) return;
		let boxSize = 1920 / 32;
		ctx.beginPath();
		ctx.lineWidth = "4";
		ctx.strokeStyle = "black";
		for (var i = 0; i < 32 * 18; i++) {
			let y = (i / 32) | 0;
			let x = i % 32;
			ctx.rect(x * boxSize, y * boxSize, boxSize, boxSize);
		}
		ctx.stroke();
	}
	
	addBlock(i, block){
		this.internal.addBlock(i, block);
		this.dirty = true;
	}
	
	exportLevel(){
		return this.internal.exportLevel();
	}
	
	setDark(val){
		this.internal.setDark(val);
	}
	
	getDark(){
		return this.internal.getDark();
	}
	
	import(data){
		this.dirty = true;
		return this.internal.import(data);
	}
}
