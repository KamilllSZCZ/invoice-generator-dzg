# Prompt 01 — utworzenie fundamentu aplikacji

Pracujesz nad nowym projektem Dual Zone Garage — lokalnym generatorem faktur dla Windows.

Najpierw przeczytaj `AGENTS.md` oraz sekcje 1–3, 7 i 9 z `PROJECT_PLAN.md`. Projekt powstaje od zera. Nie korzystaj ze starego generatora Python i nie próbuj go odtwarzać.

## Zadanie

Utwórz wyłącznie fundament aplikacji:

1. Sprawdź środowisko: Node.js, npm, Rust, Cargo i wymagania Tauri dla Windows. Nie instaluj narzędzi systemowych bez zgody. Jeżeli brakuje wymaganego narzędzia, podaj konkretny brak i zatrzymaj część zależną od niego.
2. Utwórz nowy projekt **Tauri 2 + React + TypeScript + Vite**, używając npm i aktualnych stabilnych, zgodnych wersji.
3. Nazwa aplikacji: `DualZone Garage`.
4. Identyfikator aplikacji: `pl.dualzonegarage.invoices`.
5. Przygotuj podstawowy layout: boczna nawigacja, górny pasek i główny obszar treści. Dodaj tylko widoki startowe: Pulpit, Faktury, Kontrahenci, Katalog, Ustawienia. Widoki mogą zawierać prosty komunikat, że moduł zostanie dodany w kolejnym etapie.
6. Zastosuj kierunek wizualny DZG: ciemne tło, grafitowe powierzchnie, pomarańcz `#FF6F00` i fiolet `#9C27B0`. Interfejs po polsku, czytelny i oszczędny.
7. Przygotuj sensowną strukturę katalogów, ale nie twórz pustych modułów i plików na zapas.
8. Dodaj krótki `README.md` z wymaganiami środowiska, komendami uruchamiania w trybie developerskim i budowania aplikacji.

## Ograniczenia

- Na tym etapie **nie implementuj** bazy danych, klientów, katalogu, faktur, obliczeń, PDF, backupów ani KSeF.
- Nie dodawaj przykładowych klientów ani fikcyjnych faktur.
- Nie instaluj dużych bibliotek UI ani nie twórz rozbudowanego design systemu.
- Nie dodawaj backendu HTTP ani zewnętrznych usług.
- Nie zmieniaj treści `AGENTS.md` ani `PROJECT_PLAN.md`.
- Nie wykonuj refaktoryzacji niezwiązanej z zadaniem.

## Weryfikacja i raport

Uruchom dostępne kontrole: instalację zależności, `npm run build` oraz `cargo check` dla `src-tauri`, jeżeli środowisko ma wymagane narzędzia. Uruchom aplikację developerską, jeżeli jest to możliwe.

Na końcu podaj krótko:
- co zostało utworzone,
- najważniejsze pliki i katalogi,
- komendy uruchamiania,
- wyniki wykonanych kontroli,
- brakujące wymagania środowiska lub problemy.

Nie przechodź do implementacji kolejnego etapu. Zakończ po działającym szkielecie aplikacji.
