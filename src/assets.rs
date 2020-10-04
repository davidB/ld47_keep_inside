use bevy::prelude::*;

#[derive(Default, Clone)]
pub struct AssetHandles {
    paddle_handle: Option<Handle<ColorMaterial>>,
    ball_handle: Option<Handle<ColorMaterial>>,
    font_main_handle: Option<Handle<Font>>,
}

impl AssetHandles {
    pub fn get_paddle_handle(
        &mut self,
        asset_server: &Res<AssetServer>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Handle<ColorMaterial> {
        if self.paddle_handle.is_none() {
            let bytes = include_bytes!("../assets/paddle.png");
            let asset = materials.add(
                asset_server
                    .load_from(Box::new(bytes.as_ref()))
                    .expect("load paddle.png")
                    .into(),
            );
            self.paddle_handle = Some(asset);
        };
        self.paddle_handle.unwrap()
    }

    pub fn get_ball_handle(
        &mut self,
        asset_server: &Res<AssetServer>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Handle<ColorMaterial> {
        if self.ball_handle.is_none() {
            let bytes = include_bytes!("../assets/ball.png");
            let asset = materials.add(
                asset_server
                    .load_from(Box::new(bytes.as_ref()))
                    .expect("load ball.png")
                    .into(),
            );
            self.ball_handle = Some(asset);
        };
        self.ball_handle.unwrap()
    }

    pub fn get_font_main_handle(&mut self, asset_server: &Res<AssetServer>) -> Handle<Font> {
        if self.font_main_handle.is_none() {
            let font = include_bytes!("../assets/Eduardo-Barrasa.ttf");

            let font: Handle<Font> = asset_server
                .load_from(Box::new(font.as_ref()))
                .expect("was able to load font");
            self.font_main_handle = Some(font);
        }
        self.font_main_handle.unwrap()
    }
}
