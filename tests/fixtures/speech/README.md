# Human speech for ru/en/he recognition

These are distinct human recordings from [Google FLEURS](https://huggingface.co/datasets/google/fleurs), licensed [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). Attribution: FLEURS, Google, 2022. Each original WAV was converted to mono 48 kHz Ogg/Opus at 24 kbit/s with ffmpeg, with no transcript synthesis or audio edits.

| File | Original split and filename | Reference text |
|---|---|---|
| ru.ogg | ru_ru/dev/12991791007873426296.wav | Основной религией в Молдавии является православное христианство. |
| en.ogg | en_us/dev/15158676295442294624.wav | The main local beer is 'Number One', it is not a complex beer, but pleasant and refreshing. The other local beer is called "Manta". |
| he.ogg | he_il/test/18025095235031802972.wav | מרבית הפרשנויות לדטרמיניזם טכנולוגי חולקות שני רעיונות כלליים: שפיתוח הטכנולוגיה מתקדם בדרך שלא מושפעת תרבותית או פוליטית במידה מרובה, וכי לטכנולוגיה יש "השפעות" על חברות שהן מובנות, ולא מותנות חברתית. |

Run `multilingual_344` explicitly with verified local assets as described in `docs/research/344.md`. It checks meaningful words in each original language, including readable Hebrew. The ordinary fake-Bot-API suite checks ingress, durable transcripts and delivery; it does not establish recognition quality or live Telegram acceptance. These short read-speech samples do not qualify spontaneous or noisy speech.
