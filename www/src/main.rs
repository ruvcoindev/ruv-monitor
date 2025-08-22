use yew::prelude::*;
use gloo_net::http::Request;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Clone, Debug, Default)]
struct Pool {
    id: String,
    reserves: Vec<Reserve>,
}

#[derive(Deserialize, Clone, Debug, Default)]
struct Reserve {
    asset: String,
    amount: String,
}

#[function_component(App)]
fn app() -> Html {
    let pool = use_state(|| None::<Pool>);
    let xlm_usd = use_state(|| 0.0);
    let usd_rub = use_state(|| 0.0);
    let positions = use_state(|| vec![]);

    {
        let pool = pool.clone();
        let xlm_usd = xlm_usd.clone();
        let usd_rub = usd_rub.clone();

        use_effect_with_deps(move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                loop {
                    // Пул RUV/XLM
                    let resp = Request::get("https://horizon.stellar.org/liquidity_pools/bfad578200fecbf7210294aa939d68eeb1ea9057c05673f11fedc771089bac00")
                        .send().await;
                    if let Ok(resp) = resp {
                        if resp.status().is_success() {
                            if let Ok(json) = resp.json::<Value>().await {
                                if let Ok(p) = serde_json::from_value(json) {
                                    pool.set(Some(p));
                                }
                            }
                        }
                    }

                    // Курсы
                    let cg: Value = Request::get("https://api.coingecko.com/api/v3/simple/price?ids=stellar&vs_currencies=usd")
                        .send().await.unwrap().json().await.unwrap();
                    xlm_usd.set(cg["stellar"]["usd"].as_f64().unwrap_or(0.45));

                    let cbr: Value = Request::get("https://www.cbr-xml-daily.ru/latest.js")
                        .send().await.unwrap().json().await.unwrap();
                    let rate = cbr["rates"]["USD"].as_f64().unwrap_or(0.01);
                    usd_rub.set(if rate > 0.0 { 1.0 / rate } else { 95.0 });

                    gloo_timers::future::TimeoutFuture::new(30_000).await;
                }
            });
            || ()
        }, ());
    }

    let on_load_positions = {
        let positions = positions.clone();
        let public_key = use_state(|| "".to_string());
        let on_input = {
            let pk = public_key.clone();
            Callback::from(move |e: InputEvent| {
                let value = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
                pk.set(value);
            })
        };

        Callback::from(move |_| {
            let public_key = (*public_key).clone();
            let positions = positions.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if public_key.is_empty() { return; }
                let url = format!("https://horizon.stellar.org/accounts/{}/liquidity_positions", public_key);
                let resp = Request::get(&url).send().await;
                if let Ok(resp) = resp {
                    if resp.status().is_success() {
                        let json: Value = resp.json().await.unwrap();
                        let mut list = vec![];
                        for rec in json["_embedded"]["records"].as_array().unwrap_or(&vec![]) {
                            if let Ok(pos) = serde_json::from_value(rec.clone()) {
                                list.push(pos);
                            }
                        }
                        positions.set(list);
                    }
                }
            });
        })
    };

    let p = (*pool).clone();
    let xlm = p.as_ref().and_then(|p| p.reserves.iter().find(|r| r.asset == "native")).and_then(|r| r.amount.parse::<f64>().ok()).unwrap_or(0.0);
    let ruv = p.as_ref().and_then(|p| p.reserves.iter().find(|r| r.asset.starts_with("RUV:"))).and_then(|r| r.amount.parse::<f64>().ok()).unwrap_or(0.0);
    let ruv_rub = if ruv > 0.0 { (xlm / ruv) * *xlm_usd * *usd_rub } else { 0.0 };

    html! {
        <div>
            <h1>{ "📊 ruv-monitor (web)" }</h1>
            <p><b>{ format!("1 RUV = {:.4} RUB", ruv_rub) }</b></p>

            <table>
                <tr><th>{"Актив"}</th><th>{"Количество"}</th></tr>
                <tr><td>{"RUV"}</td><td>{ format!("{:.0}", ruv) }</td></tr>
                <tr><td>{"XLM"}</td><td>{ format!("{:.2}", xlm) }</td></tr>
            </table>

            <input type="text" placeholder="Ваш публичный ключ" oninput={on_input} />
            <button onclick={on_load_positions}>{ "Загрузить позиции" }</button>

            if !(*positions).is_empty() {
                <div>
                    <h3>{ "Ваши шары" }</h3>
                    <ul>
                    { for (*positions).iter().map(|pos| {
                        let pool_id = pos["liquidity_pool"]["id"].as_str().unwrap_or("неизвестен");
                        let shares = pos["liquidity_pool"]["shares"].as_str().unwrap_or("0");
                        html! { <li>{ format!("Пул {}: {}", &pool_id[..8], shares) }</li> }
                    }) }
                    </ul>
                </div>
            }
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
