# DualZone Garage

Lokalny fundament aplikacji desktopowej do przygotowywania faktur. Interfejs działa w React + TypeScript + Vite, a okno desktopowe zapewnia Tauri 2. Na tym etapie widoki Faktury, Kontrahenci, Katalog i Ustawienia są oznaczone jako niezaimplementowane.

## Wymagania

- Windows 10/11
- Node.js i npm
- Rust stable z Cargo
- Microsoft C++ Build Tools (MSVC)
- Microsoft Edge WebView2 Runtime

Nie instaluj narzędzi systemowych automatycznie. Jeżeli brakuje Rust/Cargo albo MSVC, instalację i uruchomienie natywnej aplikacji trzeba odłożyć do czasu uzupełnienia środowiska. Interfejs można sprawdzić w przeglądarce przez Vite.

## Instalacja zależności

```bash
npm install
```

## Tryb developerski

```bash
npm run tauri dev
```

Do samego podglądu interfejsu webowego (bez natywnego okna Tauri):

```bash
npm run dev
```

## Build

```bash
npm run build
npm run tauri build
```

Build Tauri tworzy pakiet Windows i wymaga kompletu narzędzi systemowych z listy powyżej.
