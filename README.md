# ruv-monitor

Мониторинг пула RUV/XLM на блокчейне Stellar с отображением цены в рублях.

- ✅ Реальная стоимость RUV в рублях (с учётом XLM/USD и ЦБ РФ)
- ✅ Поддержка CLP-пула `bfad5782...`
- ✅ Запись истории в `ruv_history.csv`
- ✅ Веб-интерфейс на Rust + WASM (Yew)
- ✅ Поддержка публичных ключей (просмотр доли)

---

## 🖥 CLI-версия (консоль)

### Установка

```bash
cargo install wasm-pack --force

Запуск

git clone https://github.com/ruvcoindev/ruv-monitor.git
cd ruv-monitor
cargo run

Данные будут записываться в ruv_history.csv.

### 🌐 Веб-версия (браузер)

Сборка

```bash
cd www
wasm-pack build --target web

Запуск

```bash

cd pkg
python3 -m http.server 8080

Открой: http://localhost:8080

📁 Структура проекта

ruv-monitor/
├── Cargo.toml          # CLI-пакет
├── src/main.rs         # CLI: мониторинг + CSV
├── ruv_history.csv     # создаётся при запуске
├── www/
│   ├── Cargo.toml      # Web-пакет (для WASM)
│   ├── index.html
│   └── src/main.rs     # Веб-интерфейс
└── README.md

✅ Что показывает
Курс 1 RUV в рублях
Объёмы пула (RUV и XLM)
Количество участников (total_trustlines)
Возможность ввести публичный ключ и увидеть свои шары
Кнопка "Скачать CSV" (в вебе)
График роста (в разработке)

📈 График роста (в разработке)
Планируется:

Визуализация ruv_history.csv
Сравнение с инфляцией рубля
"Ты на X% богаче, чем год назад"

🤝 Спасибо
Сервис использует:

Stellar Horizon API
cbr-xml-daily.ru — для курсов ЦБ РФ
CoinGecko API — для XLM/USD

📄 Лицензия
MIT