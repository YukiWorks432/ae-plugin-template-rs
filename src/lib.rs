use after_effects as ae;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
  Example,
}

#[derive(Default)]
struct Plugin {}

ae::define_effect!(Plugin, (), Params);

trait ToPixel {
  fn to_pixel32(&self) -> PixelF32;
  fn to_pixel16(&self) -> Pixel16;
  fn to_pixel8(&self) -> Pixel8;
}
impl ToPixel for Pixel8 {
  fn to_pixel32(&self) -> PixelF32 {
    PixelF32 {
      red: self.red as f32 / ae::MAX_CHANNEL8 as f32,
      green: self.green as f32 / ae::MAX_CHANNEL8 as f32,
      blue: self.blue as f32 / ae::MAX_CHANNEL8 as f32,
      alpha: self.alpha as f32 / ae::MAX_CHANNEL8 as f32,
    }
  }
  fn to_pixel16(&self) -> Pixel16 {
    Pixel16 {
      red: (self.red as f32 / ae::MAX_CHANNEL8 as f32
        * ae::MAX_CHANNEL16 as f32) as u16,
      green: (self.green as f32 / ae::MAX_CHANNEL8 as f32
        * ae::MAX_CHANNEL16 as f32) as u16,
      blue: (self.blue as f32 / ae::MAX_CHANNEL8 as f32
        * ae::MAX_CHANNEL16 as f32) as u16,
      alpha: (self.alpha as f32 / ae::MAX_CHANNEL8 as f32
        * ae::MAX_CHANNEL16 as f32) as u16,
    }
  }
  fn to_pixel8(&self) -> Pixel8 {
    *self
  }
}
impl ToPixel for Pixel16 {
  fn to_pixel32(&self) -> PixelF32 {
    PixelF32 {
      red: self.red as f32 / ae::MAX_CHANNEL16 as f32,
      green: self.green as f32 / ae::MAX_CHANNEL16 as f32,
      blue: self.blue as f32 / ae::MAX_CHANNEL16 as f32,
      alpha: self.alpha as f32 / ae::MAX_CHANNEL16 as f32,
    }
  }
  fn to_pixel16(&self) -> Pixel16 {
    *self
  }
  fn to_pixel8(&self) -> Pixel8 {
    Pixel8 {
      red: (self.red as f32 / ae::MAX_CHANNEL16 as f32
        * ae::MAX_CHANNEL8 as f32) as u8,
      green: (self.green as f32 / ae::MAX_CHANNEL16 as f32
        * ae::MAX_CHANNEL8 as f32) as u8,
      blue: (self.blue as f32 / ae::MAX_CHANNEL16 as f32
        * ae::MAX_CHANNEL8 as f32) as u8,
      alpha: (self.alpha as f32 / ae::MAX_CHANNEL16 as f32
        * ae::MAX_CHANNEL8 as f32) as u8,
    }
  }
}
impl ToPixel for PixelF32 {
  fn to_pixel32(&self) -> PixelF32 {
    *self
  }
  fn to_pixel16(&self) -> Pixel16 {
    Pixel16 {
      red: (self.red.clamp(0.0, 1.0) * ae::MAX_CHANNEL16 as f32) as u16,
      green: (self.green.clamp(0.0, 1.0) * ae::MAX_CHANNEL16 as f32) as u16,
      blue: (self.blue.clamp(0.0, 1.0) * ae::MAX_CHANNEL16 as f32) as u16,
      alpha: (self.alpha.clamp(0.0, 1.0) * ae::MAX_CHANNEL16 as f32) as u16,
    }
  }
  fn to_pixel8(&self) -> Pixel8 {
    Pixel8 {
      red: (self.red.clamp(0.0, 1.0) * ae::MAX_CHANNEL8 as f32) as u8,
      green: (self.green.clamp(0.0, 1.0) * ae::MAX_CHANNEL8 as f32) as u8,
      blue: (self.blue.clamp(0.0, 1.0) * ae::MAX_CHANNEL8 as f32) as u8,
      alpha: (self.alpha.clamp(0.0, 1.0) * ae::MAX_CHANNEL8 as f32) as u8,
    }
  }
}

trait Add {
  fn add(&self, value: f32) -> Self;
}
impl Add for PixelF32 {
  fn add(&self, value: f32) -> Self {
    PixelF32 {
      red: (self.red + value / 100.0).clamp(0.0, 1.0),
      green: (self.green + value / 100.0).clamp(0.0, 1.0),
      blue: (self.blue + value / 100.0).clamp(0.0, 1.0),
      alpha: self.alpha,
    }
  }
}

impl AdobePluginGlobal for Plugin {
  fn can_load(_host_name: &str, _host_version: &str) -> bool {
    true
  }

  fn params_setup(
    &self,
    params: &mut ae::Parameters<Params>,
    _in_data: InData,
    _: OutData,
  ) -> Result<(), Error> {
    params.add(
      Params::Example,
      "Example",
      ae::FloatSliderDef::setup(|f| {
        f.set_valid_min(0.0);
        f.set_valid_max(100.0);
        f.set_slider_min(0.0);
        f.set_slider_max(100.0);
        f.set_default(50.0);
        f.set_precision(2);
        f.set_display_flags(ae::ValueDisplayFlag::PERCENT);
      }),
    )?;

    Ok(())
  }

  fn handle_command(
    &mut self,
    cmd: ae::Command,
    in_data: InData,
    mut out_data: OutData,
    params: &mut ae::Parameters<Params>,
  ) -> Result<(), ae::Error> {
    let get_params = || {
      let example =
        params.get(Params::Example)?.as_float_slider()?.value() as f32;

      Ok::<_, Error>(example)
    };
    match cmd {
      ae::Command::About => {
        out_data
          .set_return_msg("{{display-name}}\r{{copyright}}\r{{description}}");
      }
      ae::Command::GlobalSetup => {
        // For Premiere - declare supported pixel formats
        if in_data.is_premiere() {
          let suite = ae::pf::suites::PixelFormat::new()?;

          // Add the pixel formats we support in order of preference.
          suite.clear_supported_pixel_formats(in_data.effect_ref())?;
          let formats = [
            ae::pr::PixelFormat::Bgra4444_32f,
            ae::pr::PixelFormat::Bgra4444_16u,
            ae::pr::PixelFormat::Bgra4444_8u,
          ];
          for x in formats {
            suite.add_supported_pixel_format(in_data.effect_ref(), x)?;
          }
        }
      }
      ae::Command::Render {
        in_layer,
        mut out_layer,
      } => {
        let progress_final = out_layer.height() as _;

        let example = get_params()?;

        in_layer.iterate_with(
          &mut out_layer,
          0,
          progress_final,
          None,
          |_x: i32,
           _y: i32,
           pixel: ae::GenericPixel,
           out_pixel: ae::GenericPixelMut|
           -> Result<(), Error> {
            match (pixel, out_pixel) {
              (
                ae::GenericPixel::Pixel8(pixel),
                ae::GenericPixelMut::Pixel8(out_pixel),
              ) => *out_pixel = pixel.to_pixel32().add(example).to_pixel8(),
              (
                ae::GenericPixel::Pixel16(pixel),
                ae::GenericPixelMut::Pixel16(out_pixel),
              ) => *out_pixel = pixel.to_pixel32().add(example).to_pixel16(),
              _ => return Err(Error::BadCallbackParameter),
            }
            Ok(())
          },
        )?;
      }
      ae::Command::SmartPreRender { mut extra } => {
        let req = extra.output_request();

        if let Ok(in_result) = extra.callbacks().checkout_layer(
          0,
          0,
          &req,
          in_data.current_time(),
          in_data.time_step(),
          in_data.time_scale(),
        ) {
          let _ = extra.union_result_rect(in_result.result_rect.into());
          let _ = extra.union_max_result_rect(in_result.max_result_rect.into());
        } else {
          return Err(Error::InterruptCancel);
        }
      }
      ae::Command::SmartRender { extra } => {
        let cb = extra.callbacks();
        let input_world = cb.checkout_layer_pixels(0)?;
        let mut output_world = cb.checkout_output()?;

        let progress_final = output_world.height() as _;

        let example = get_params()?;

        input_world.iterate_with(
          &mut output_world,
          0,
          progress_final,
          None,
          |_x: i32,
           _y: i32,
           pixel: ae::GenericPixel,
           out_pixel: ae::GenericPixelMut|
           -> Result<(), Error> {
            match (pixel, out_pixel) {
              (
                ae::GenericPixel::Pixel8(pixel),
                ae::GenericPixelMut::Pixel8(out_pixel),
              ) => *out_pixel = pixel.to_pixel32().add(example).to_pixel8(),
              (
                ae::GenericPixel::Pixel16(pixel),
                ae::GenericPixelMut::Pixel16(out_pixel),
              ) => *out_pixel = pixel.to_pixel32().add(example).to_pixel16(),
              (
                ae::GenericPixel::PixelF32(pixel),
                ae::GenericPixelMut::PixelF32(out_pixel),
              ) => *out_pixel = pixel.add(example),
              _ => return Err(Error::BadCallbackParameter),
            }
            Ok(())
          },
        )?;

        cb.checkin_layer_pixels(0)?;
      }
      _ => {}
    }
    Ok(())
  }
}
