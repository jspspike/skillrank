use worker::*;

mod matchmaking;
mod rankings;
mod utils;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::set_panic_hook();
    /*let router = Router::new();

    router
        .get("/", |_, _| Response::ok("Hello from Workers!"))
        .run(req, env)
        .await*/

    let namespace = env.durable_object("RANKINGS")?;
    let stub = namespace.id_from_name("Spikeball")?.get_stub()?;
    stub.fetch_with_request(req).await
}
