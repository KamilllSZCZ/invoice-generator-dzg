# DualZone Garage

Lokalny generator dokumentów rozwijany w Tauri 2, React, TypeScript, Rust i SQLite. Panel **Ustawienia** jest dostępny; Faktury, Kontrahenci i Katalog pozostają oznaczone jako moduły w przygotowaniu.

## Uruchomienie i weryfikacja

Wymagania: Windows 10/11, Node.js z npm, Rust stable, Microsoft C++ Build Tools (MSVC) i WebView2 Runtime.

```powershell
npm ci
npm run tauri dev
```

```powershell
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

`npm run dev` uruchamia sam frontend. Trwałe ustawienia i natywne okna wyboru plików wymagają Tauri.

## Panel Ustawienia

- **Dane sprzedawcy:** nazwa, adres, kontakt, strona internetowa i rachunek bankowy. Pola są opcjonalne. Logo PNG/JPEG jest sprawdzane przez dekoder, maksymalnie 10 MiB i 25 mln pikseli. Zmiana logo zapisuje się od razu, pozostałe pola przez przycisk zapisu.
- **Dokumenty:** opcjonalna waluta, termin i metoda płatności, miejsce wystawienia oraz uwagi. Nie są podstawiane dane rejestrowe, status VAT ani stawki podatku. Formularz rejestrowy/podatkowy nie należy do tego etapu; istniejące opcjonalne i nieznane pola modelu pozostają zachowane.
- **Numeracja:** seria ogólnego typu `invoice`, tokeny `{prefix}`, `{year}`, `{month}`, `{counter}` i reset roczny, miesięczny lub brak resetu. Podgląd dotyczy zapisanej konfiguracji, nie rezerwuje numeru i może się zmienić. Przydział numeru musi być częścią tej samej transakcji co przyszły zapis dokumentu i jego migawki. Nie ma samodzielnej komendy rezerwującej numer.
- **Wygląd:** dwa warianty z działającym podglądem, widoczność danych i stopka. Podgląd jest oznaczony jako wzór z danymi przykładowymi. To przygotowanie ustawień dla przyszłego modułu dokumentów, nie eksport PDF.
- **Aplikacja:** zapisany motyw systemowy/jasny/ciemny, gęstość interfejsu i rzeczywista wersja pakietu.
- **Kopie zapasowe:** eksport, sprawdzenie i przywrócenie pełnego lokalnego zestawu danych.

Każda sekcja zapisuje się niezależnie i atomowo przez jawne komendy Rust. Szkice pozostają po przełączeniu sekcji; opuszczenie panelu z niezapisanymi zmianami wymaga potwierdzenia. Edycja sprzedawcy nie modyfikuje historycznych migawek dokumentów.

## Lokalizacja danych

Baza znajduje się w katalogu użytkownika wyznaczonym przez Tauri, poza folderem instalacji i źródłami. W standardowej konfiguracji Windows jest to `%APPDATA%\pl.dualzonegarage.invoices`:

```text
pl.dualzonegarage.invoices/
  instance.lock
  data/
    app.sqlite
    assets/
  recovery/
```

Dokładna ścieżka jest widoczna w **Ustawienia → Kopie zapasowe**. Przycisk otwarcia folderu może otworzyć wyłącznie zarządzany katalog danych aplikacji. Jedna instancja programu może korzystać z danego profilu; osobne profile testowe mają osobne identyfikatory i blokady.

Schemat jest aktualizowany transakcyjnie migracjami `0001` i `0002`. Nowsza nieobsługiwana wersja bazy jest odrzucana. Dane pozostają w tym samym katalogu przy aktualizacji aplikacji.

## Tworzenie kopii i przywracanie

1. Wybierz **Utwórz kopię** i wskaż lokalny katalog poza aktywnym `data/`. Samodzielny ZIP zawiera spójny obraz całej bazy, wszystkie zarządzane zasoby i manifest SHA-256. Nazwa ma czas UTC. Kolizja dodaje `-01`, `-02` itd. bez nadpisywania istniejących plików.
2. Wybierz **Wybierz kopię do przywrócenia**. Aplikacja sprawdzi rzeczywiste wpisy ZIP, ścieżki i limity, sumy kontrolne, integralność i wymagany schemat SQLite, dane ustawień oraz powiązane logo.
3. Sprawdź podsumowanie. Zaznacz zgodę, następnie wybierz **Zastąp wszystkie lokalne dane**. Operacja zastępuje całą bazę i zasoby oraz odrzuca szkice formularzy. Samo wybranie kopii niczego nie zmienia.
4. Backend ponownie sprawdza kopię i porównuje jej odcisk z potwierdzonym plikiem. Podmiana archiwum wymaga ponownego sprawdzenia. Przed zmianą powstaje zweryfikowana kopia bieżącego stanu w `recovery/`; jej ścieżka jest pokazywana po sukcesie.

Nieudana podmiana wycofuje cały zestaw bazy i zasobów. Po przerwaniu procesu w trakcie podmiany następne uruchomienie odtwarza poprzedni kompletny katalog zamiast tworzyć pustą bazę. Niedokończone dane są zachowane w `recovery/interrupted-*`. Kopie odzyskiwania nie mają automatycznej retencji.

Format kopii v1 używa ZIP32, bez szyfrowania, ZIP64, archiwów wieloczęściowych i deskryptorów strumieniowych. Eksporter generuje obsługiwany format. Limity: 512 MiB archiwum, 1 GiB po rozpakowaniu, 512 MiB na wpis, 1024 wpisy, kompresja do 100:1. Dane nie są wysyłane do usług zewnętrznych. SHA-256 wykrywa zmianę/uszkodzenie pliku, lecz nie jest podpisem autora.

## Obecne ograniczenia

Panel nie wystawia faktur i nie generuje PDF. Pozostałe moduły, KSeF, synchronizacja chmurowa, logowanie i księgowość są poza zakresem. Numeracja udostępnia warstwę transakcyjną dla przyszłego wystawiania. Podgląd używa lokalnej daty systemowej, a nazwy i manifesty kopii czasu UTC.

Rust sprawdza migracje, zapis po ponownym otwarciu, opcjonalny profil, niezmienność migawek, numerację i współbieżność, pełne kopie, walidację, wycofanie i odzyskiwanie. Vitest/RTL sprawdza formularze, błędy, szkice, preferencje i jawne potwierdzanie kopii.

## Wynik weryfikacji panelu (2026-11-14)

- `npm test`: 17/17 testów, siedem plików.
- `npm run build`: TypeScript i produkcyjny frontend Vite bez błędów.
- `cargo test --manifest-path src-tauri/Cargo.toml --offline`: 46/46 testów Rust.
- `cargo check --manifest-path src-tauri/Cargo.toml --offline`: bez błędów. Środowisko nie obsługuje hardlinków cache; wyłączenie kompilacji przyrostowej usuwa ostrzeżenie bez zmiany kodu.
- `npm run tauri build -- --debug --no-bundle --config <konfiguracja-testowa>`: natywny build Windows bez instalatora. Profil ma osobny identyfikator, więc nie używa właściwych danych aplikacji.
- Natywne okno uruchomiło się, odczytało ustawienia przez IPC i pozwoliło edytować testowy profil. Pełny ręczny scenariusz zapisu po restarcie oraz eksportu/przywracania przez systemowe dialogi pozostaje do wykonania: sterowanie oknem przerywało operacje błędami geometrii i aktywnego procesu. Odpowiadające scenariusze usług i formularzy przechodzą testy automatyczne.

Instalator Windows i wydanie produkcyjne nie były testowane w tym etapie.
