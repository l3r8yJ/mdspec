## Эндпоинт: Unicode

Path: `GET /unicode`

### Логика работы

1. Проверить [первую][one], [вторую][two], [текущую][self] и [ресурс][query].
2. Проверить [изображение][image] и [вложенный файл][nested].
3. Оставить литералы \[не ссылка\] и `[not a link][missing]`.

### Ссылки

[one]: ./target.md#%D0%BF%D1%80%D0%BE%D0%B2%D0%B5%D1%80%D0%BA%D0%B0-%D0%BF%D1%80%D0%B0%D0%B2
[two]: target.md#пример-кода
[self]: #логика-работы
[query]: target.md?download=1#проверка-прав
[image]: image.svg#arbitrary-anchor
[nested]: nested/../nested/%D0%B8%D0%BC%D1%8F%20%D1%84%D0%B0%D0%B9%D0%BB%D0%B0.md#проверка-прав
