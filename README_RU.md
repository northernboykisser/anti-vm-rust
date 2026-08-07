# 🛡️ anti-vm

Rust-библиотека для защиты программы от запуска в "нежелательной" среде.

---

## Возможности

| Проверка | Что делает |
|---|---|
| 🌍 **IP-фильтр** | Завершает, если публичный IP принадлежит стране ЕС или НАТО |
| 🔗 **HTTP-фильтр** | Завершает, если указанный URL вернул `200 OK` |
| 💻 **VM-фильтр** | Завершает при обнаружении драйверов VirtualBox или VMware |
| 📡 **Сеть** | Завершает, если интернет недоступен |
| 🖥️ **Экран** | Завершает, если разрешение экрана равно 800×600 |
| ⚙️ **CPU** | Завершает, если доступно только 1 логическое ядро |
| 🧠 **RAM** | Завершает, если оперативной памяти меньше 4 ГБ |

---

## Установка

Добавь зависимость в `Cargo.toml` своего проекта:

```toml
[dependencies]
anti-vm = "0.1"
```

---

## Быстрый старт

```rust
use anti_vm::ProtectionBuilder;

fn main() {
    // Запускаем все проверки
    ProtectionBuilder::new()
        .set_vm(true)
        .set_network(true)
        .set_screen(true)
        .set_cpu(true)
        .set_ram(true)
        .set_ip(true)
        .set_http(true)
        .init(); // <-- если проверка не прошла, программа молча закроется здесь

    // Сюда код дойдёт только если всё в порядке
    println!("Добро пожаловать!");
    // ... остальной код программы
}
```

---

## Настройка фильтров

Каждый фильтр можно **включить** (`true`) или **отключить** (`false`) независимо от остальных.

### `.set_vm(bool)` — Проверка виртуальной машины

Ищет файлы драйверов VirtualBox и VMware в системных директориях Windows.

```rust
ProtectionBuilder::new()
    .set_vm(true)  // включено по умолчанию
    .init();
```

**Что проверяется:**

| VirtualBox | VMware |
|---|---|
| `VBoxGuest.sys` | `vmxnet3.sys` |
| `VBoxVideo.sys` | `vm3d.sys` |
| `VBoxWddm.sys` | `vmwvxpe.sys` |
| `VBoxSF.sys` | `vmmemctl.sys` |
| `VBoxMouse.sys` | `vmci.sys` |
| `VBoxService.exe` | `vmhgfs.sys` |
| | `vmvss.sys` |
| | `pvscsi.sys` |
| | `vmblock.sys` |

**Директории поиска:**
- `C:\Windows\System32\drivers`
- `C:\Windows\System32`
- `C:\Windows\SysWOW64\drivers`
- `C:\Windows\SysWOW64`

---

### `.set_network(bool)` — Проверка наличия интернета

Пробует установить TCP-соединение с `8.8.8.8:53` (Google DNS). Если не удалось — завершает процесс.

```rust
ProtectionBuilder::new()
    .set_network(true)  // включено по умолчанию
    .init();
```
---

### `.set_screen(bool)` — Проверка разрешения экрана

Читает разрешение основного монитора через Windows API (`GetSystemMetrics`). Если ширина = 800 и высота = 600 — завершает. Такое разрешение характерно для виртуальных машин без установленных гостевых дополнений.

```rust
ProtectionBuilder::new()
    .set_screen(true)  // включено по умолчанию
    .init();
```

---

### `.set_cpu(bool)` — Проверка количества ядер CPU

Определяет количество логических ядер процессора. Если их ≤ 1 — завершает. Виртуальные машины нередко создаются с 1 ядром.

```rust
ProtectionBuilder::new()
    .set_cpu(true)  // включено по умолчанию
    .init();
```

---

### `.set_ram(bool)` — Проверка объёма ОЗУ

Считывает суммарный объём оперативной памяти. Если меньше **4 ГБ** — завершает.

```rust
ProtectionBuilder::new()
    .set_ram(true)  // включено по умолчанию
    .init();
```

---

### `.set_ip(bool)` — Проверка IP-адреса по геолокации

```rust
ProtectionBuilder::new()
    .set_ip(true)  // включено по умолчанию
    .init();
```

**Разрешённые страны (СНГ):**
`RU` `BY` `UA` `KZ` `TJ` `UZ` `KG` `AM` `AZ` `GE` `MD` `TM`

**Заблокированные страны:**

<details>
<summary>Показать полный список (ЕС + НАТО)</summary>

**ЕС:** AT BE BG HR CY CZ DK EE FI FR DE GR HU IE IT LV LT LU MT NL PL PT RO SK SI ES SE

**НАТО (не в ЕС):** AL CA IS ME MK NO TR GB US

</details>

---

### `.set_http(bool)` — HTTP-проверка по внешнему URL

Выполняет GET-запрос на заданный URL. Если сервер вернул `200 OK` — завершает. Применяется как дополнительный «выключатель»: если твой сервер вернёт 200, программа закроется.

```rust
ProtectionBuilder::new()
    .set_http(true)  // включено по умолчанию
    .init();
```

---

## Дополнительные параметры

### Изменить URL для HTTP-проверки

```rust
ProtectionBuilder::new()
    .set_http(true)
    .http_url("http://my-kill-switch.example.com")
    .init();
```

### Изменить URL сервиса геолокации

```rust
ProtectionBuilder::new()
    .set_ip(true)
    .ip_api_url("http://ip-api.com/json/?fields=status,countryCode")
    .init();
```

### Изменить таймаут сетевых запросов

```rust
ProtectionBuilder::new()
    .timeout(3)  // 3 секунды
    .init();
```

---

## Порядок выполнения проверок

```
1. VM-драйверы    — локально, мгновенно
2. Экран          — локально, мгновенно  
3. CPU            — локально, мгновенно
4. RAM            — локально, мгновенно
──────────────────────────────────────
5. Сеть           — нужен интернет
6. IP-геолокация  — нужен интернет
7. HTTP-запрос    — нужен интернет
```
