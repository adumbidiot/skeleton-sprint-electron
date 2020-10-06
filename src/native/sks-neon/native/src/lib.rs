mod util;

use neon::prelude::*;
use sks::*;

fn export_1d_patch(mut cx: FunctionContext) -> JsResult<JsString> {
    let v = cx.argument::<JsArray>(0)?;
    let v_str = v.to_string(&mut cx)?;
    Ok(v_str)
}

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("hello sks-neon"))
}

/// Converts from the lvlbuilders old internal rep into lbl format
fn encode_block_lbl(mut cx: FunctionContext) -> JsResult<JsValue> {
    let block_str = cx.argument::<JsString>(0)?.value();
    match util::builder_internal_to_block(&block_str) {
        Some(v) => Ok(cx.string(v.as_lbl()).upcast()),
        None => {
            println!("[sks_rust::encode_block_lbl] Unknown: {}", &block_str);
            Ok(cx.null().upcast())
        }
    }
}

fn block_array_to_js_array<'a, T: neon::object::This>(
    mut cx: CallContext<'a, T>,
    blocks: &[Block],
) -> JsResult<'a, JsValue> {
    let js_array = JsArray::new(&mut cx, blocks.len() as u32);
    for (i, block) in blocks.iter().enumerate() {
        let data_str = util::block_to_builder_internal(&block);
        let string = cx.string(data_str);
        js_array.set(&mut cx, i as u32, string)?;
    }
    Ok(js_array.upcast())
}

fn get_frame<'a>(
    cx: &mut CallContext<'a, JsLevelBuilder>,
) -> Result<Handle<'a, JsValue>, neon::result::Throw> {
    let mut this = cx.this();

    let img_data = {
        let guard = cx.lock();
        let mut lvlbuilder = this.borrow_mut(&guard);
        lvlbuilder.skeleton_sprint_levelbuilder.get_raw_image()
    };

    let img_data = match img_data {
        Ok(data) => data,
        Err(e) => return cx.throw_error(&e.to_string()),
    };

    let img_data = util::rgba_image_to_image_data(cx, img_data)?;
    let canvas = util::img_data_to_canvas(cx, img_data)?;

    Ok(canvas.upcast())
}

fn get_image<'a>(
    cx: &mut CallContext<'a, JsLevelBuilder>,
) -> Result<Handle<'a, JsValue>, neon::result::Throw> {
    let mut this = cx.this();

    let img_data = {
        let guard = cx.lock();
        let mut lvlbuilder = this.borrow_mut(&guard);
        lvlbuilder.render_image()
    };

    let img_data = match img_data {
        Ok(data) => data.into_rgba(),
        Err(_e) => return cx.throw_error("SKS render error"),
    };

    let img_data = util::rgba_image_to_image_data(cx, img_data)?;
    let canvas = util::img_data_to_canvas(cx, img_data)?;

    Ok(canvas.upcast())
}

declare_types! {
    pub class JsLevelBuilder for LevelBuilder {
        init(_cx) {
            Ok(sks::LevelBuilder::new())
        }

        method getFrame(mut cx) {
            get_frame(&mut cx)
        }

        method getImage(mut cx) {
            get_image(&mut cx)
        }

        method setGrid(mut cx) {
            let grid = cx.argument::<JsBoolean>(0)?.value();
            let mut this = cx.this();
            {
                let guard = cx.lock();
                let mut lvlbuilder = this.borrow_mut(&guard);
                lvlbuilder.set_grid(grid);
            }

            Ok(cx.undefined().upcast())
        }

        method getLevelData(mut cx) {
            let this = cx.this();
            let level_data = {
                let guard = cx.lock();
                let lvlbuilder = this.borrow(&guard);
                let data = lvlbuilder.get_level_data().to_vec();
                drop(lvlbuilder);
                block_array_to_js_array(cx, &data)?
            };

            Ok(level_data)
        }

        method addBlock(mut cx) {
            let i = cx.argument::<JsNumber>(0)?.value() as usize;
            let block = cx.argument::<JsString>(1)?.value();
            let mut this = cx.this();
            let _msg = {
                let guard = cx.lock();
                let mut lvlbuilder = this.borrow_mut(&guard);
                lvlbuilder.add_block(i, util::builder_internal_to_block(&block).unwrap());
            };

            Ok(cx.null().upcast())
        }

        method export(mut cx) {
            let block = cx.argument_opt(0).map(|v| Ok(v.to_string(&mut cx)?.value())).transpose()?;
            let block = block.unwrap_or_else(|| "lbl".into());

            let format = match block.as_str() {
                "as3" => {
                    sks::FileFormat::As3
                },
                "lbl" => {
                    sks::FileFormat::Lbl
                },
                o => {
                    return cx.throw_error(&format!("Unknown Option: {}", o));
                }
            };

             let mut this = cx.this();

             let ret = {
                let guard = cx.lock();
                let lvlbuilder = this.borrow_mut(&guard);
                lvlbuilder.export_format(&format).unwrap()
            };
            Ok(cx.string(&ret).upcast())
        }

        method exportLevel(mut cx){
            let this = cx.this();
            let level_data = {
                let guard = cx.lock();
                let lvlbuilder = this.borrow(&guard);
                let data = lvlbuilder.export().unwrap();
                drop(lvlbuilder);
                block_array_to_js_array(cx, &data)?
            };

            Ok(level_data)
        }

        method setDark(mut cx){
            let val = cx.argument::<JsBoolean>(0)?.value();
            let mut this = cx.this();
            {
                let guard = cx.lock();
                let mut lvlbuilder = this.borrow_mut(&guard);
                lvlbuilder.set_dark(val);
            }
            Ok(cx.undefined().upcast())
        }

        method getDark(mut cx){
            let this = cx.this();
            let dark = {
                let guard = cx.lock();
                let lvlbuilder = this.borrow(&guard);
                lvlbuilder.get_dark()
            };
            Ok(cx.boolean(dark).upcast())
        }

        method import(mut cx){
            let val = cx.argument::<JsString>(0)?.value();
            let (level_num, data) = match sks::decode(&val) {
                Ok(d) => d,
                Err(_e) => return Ok(cx.boolean(false).upcast()),
            };

            let mut this = cx.this();
            {
                let guard = cx.lock();
                let mut lvlbuilder = this.borrow_mut(&guard);
                if let Some(v) = level_num {
                    lvlbuilder.set_level(v);
                }
                lvlbuilder.import(&data);
            }
            Ok(cx.boolean(true).upcast())
        }

        method setLevel(mut cx){
            let arg = cx.argument::<JsValue>(0)?;
            let level_num = if let Ok(v) = arg.downcast::<JsNumber>() {
                LevelNum::Num(v.value() as usize)
            }else if let Ok(v) = arg.downcast::<JsString>() {
                LevelNum::String(v.value())
            }else{
                return cx.throw_error("Invalid Arg");
            };

            let mut this = cx.this();
            {
                let guard = cx.lock();
                let mut lvlbuilder = this.borrow_mut(&guard);

                lvlbuilder.set_level(level_num);
            }

            Ok(cx.undefined().upcast())
        }
    }
}

register_module!(mut cx, {
    cx.export_function("hello", hello)?;
    cx.export_function("export1DPatch", export_1d_patch)?;
    cx.export_function("encodeBlockLBL", encode_block_lbl)?;

    cx.export_class::<JsLevelBuilder>("LevelBuilder")
});
