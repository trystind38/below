use bevy::{prelude::*, window::PresentMode};

const TITLE: &str = "bv02 Basic";
const WIN_W: u32 = 1280;
const WIN_H: u32 = 720;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 1.0, 1.0)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: TITLE.into(),
                resolution: (WIN_W, WIN_H).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, slideshow)
        .run();
}

#[derive(Resource)]
struct Slideshow {
    images: Vec<Handle<Image>>,
    texts: Vec<String>,
    current: usize,
    timer: Timer,
}

#[derive(Component)]
struct SlideshowImage;

#[derive(Component)]
struct SlideshowText;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    //TO IMPLEMENT YOUR OWN IMAGE:
    //1. place the image in the 'assets' folder. make sure it is a PNG, and titling it with your name would be useful.
    //2. go into the images vec below and paste 'asset_server.load("yourImage.png"),'
    //3. at the same index (this is important), write your name in the texts vector.

    //this is all that you need to do. test the code to make sure it runs before you submit a pull request.

    let images = vec![
        asset_server.load("NicholasMyers.png"),
        asset_server.load("NickCheddar.png"),
        asset_server.load("TrystinDeRemer.png"),
        asset_server.load("CalebSarmiento.png"),
    ];

    let texts = vec![
        "Nicholas 'Merlin' Myers".to_string(),
        "Nick Cheddar".to_string(),
        "Trystin DeRemer".to_string(),
        "Caleb Sarmiento".to_string(),
    ];

    commands.spawn((
        Sprite::from_image(images[0].clone()),
        SlideshowImage,
    ));

    commands.spawn((
        Text::new(texts[0].clone()),
        SlideshowText,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        },
    ));

    commands.insert_resource(Slideshow {
        images,
        texts,
        current: 0,
        timer: Timer::from_seconds(2.0, TimerMode::Repeating),
    });
}

fn slideshow(
    time: Res<Time>,
    mut slideshow: ResMut<Slideshow>,
    mut image: Query<&mut Sprite, With<SlideshowImage>>,
    mut text: Query<&mut Text, With<SlideshowText>>,
) {
    slideshow.timer.tick(time.delta());

    if slideshow.timer.just_finished() {
        slideshow.current =
            (slideshow.current + 1) % slideshow.images.len();

        for mut sprite in &mut image {
            *sprite = Sprite::from_image(
                slideshow.images[slideshow.current].clone(),
            );
        }

        for mut text in &mut text {
            **text = slideshow.texts[slideshow.current].clone();
        }
    }
}