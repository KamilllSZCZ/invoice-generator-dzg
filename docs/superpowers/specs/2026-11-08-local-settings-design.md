# Ustawienia lokalne — projekt

**Data:** 2026-11-08

**Status:** projekt do przeglądu; implementacja nie jest zatwierdzona

**Zakres:** lokalne ustawienia aplikacji DualZone Garage oraz zarządzanie jej danymi i kopiami zapasowymi

## 1. Cel i ustalenia

Rozbudować istniejący widok Ustawienia w działający panel konfiguracyjny. Dane muszą być trwałe, lokalne, dostępne dla kolejnych modułów aplikacji i zachowywać się przewidywalnie po ponownym uruchomieniu.

Uzgodnione wymagania:

- Ustawienia będą przechowywane w SQLite w katalogu danych użytkownika, a dostęp do bazy będzie realizowany wyłącznie przez Rust/Tauri.
- Frontend będzie korzystał z jawnych, typowanych komend Tauri; nie będzie wykonywał dowolnego SQL.
- Status VAT, stawki podatkowe, NIP, REGON i pozostałe dane rejestrowe pozostają opcjonalne i bez wartości początkowych. Model przewidzi ich późniejsze dodanie.
- Zmiana ustawień firmy nie może zmieniać danych zapisanych w migawkach już wystawionych dokumentów.
- Numeracja ma być konfigurowalna i odporna na duplikaty; przydzielenie numeru oraz zapis wystawianego dokumentu będą jedną transakcją.
- Przywrócenie kopii zastępuje całą bieżącą bazę tylko po wyraźnym potwierdzeniu użytkownika. Przed podmianą tworzona jest automatyczna kopia obecnego stanu.
- Kopia uszkodzona, niekompletna lub niezgodna z obsługiwanym formatem nie może zmienić bieżących danych.
- Dane firmy, klientów i dokumentów nie opuszczają urządzenia.

## 2. Zakres panelu

Panel będzie podzielony na sekcje z prostą nawigacją i niezależnym zapisem:

### Dane firmy

- nazwa firmy używana na dokumentach;
- adres: ulica i numer, kod pocztowy, miejscowość, kraj;
- dane kontaktowe: e-mail, telefon i opcjonalnie strona internetowa;
- numer rachunku bankowego;
- logo przechowywane lokalnie jako plik PNG lub JPEG w zarządzanym katalogu zasobów aplikacji; baza przechowuje wyłącznie jego identyfikator względny;
- opcjonalne pola rozszerzeń rejestrowych i podatkowych, w tym NIP, REGON, status VAT oraz konfiguracja stawek — niewymagane, bez wstępnie przyjętych wartości.

Przy wczytywaniu logo backend sprawdza rzeczywisty typ pliku oraz ogranicza rozmiar pliku i wymiary obrazu; wartości limitów zostaną ustalone w planie implementacji. Zasób nie może wskazywać ścieżki poza katalogiem aplikacji.

### Domyślne ustawienia dokumentów

- waluta;
- domyślny termin płatności;
- metoda płatności;
- miejsce wystawienia;
- domyślne uwagi.

Pola, które wymagają decyzji użytkownika, nie będą wymuszane przy konfiguracji firmy. Jeśli przyszły proces tworzenia dokumentu będzie wymagał wartości, ma zwrócić konkretną informację, czego brakuje, zamiast podstawiać niepotwierdzone dane.

### Numeracja

- konfiguracja wzoru numeru, prefiksu, uwzględniania roku i miesiąca oraz licznika;
- konfiguracja przypisana do typu dokumentu, z modelem rozszerzalnym o przyszłe typy;
- interfejs pokazuje tylko typy dokumentów obsługiwane przez aplikację — nie tworzy pozornych ustawień dla niezaimplementowanych modułów;
- licznik i jego zakres/reset są przechowywane w bazie; aplikacja waliduje wzór i zakres przed zapisem.

Wystawienie dokumentu będzie pobierać konfigurację, przydzielać numer i zapisywać niezmienną migawkę sprzedawcy/nabywcy w jednej transakcji. Unikalne ograniczenie bazy będzie ostatnią ochroną przed kolizją. Zmiana wzoru nie zmienia numerów dokumentów już zapisanych.

### Wygląd dokumentów

- ustawienie stopki;
- widoczność poszczególnych danych firmy na dokumencie;
- wybór dostępnego szablonu;
- logo współdzielone z danymi firmy;
- wybór jednego z wbudowanych wariantów wyglądu udostępnionych przez panel;
- podgląd przykładowego dokumentu odzwierciedlający bieżące ustawienia.

Panel udostępnia wyłącznie warianty, dla których istnieje podgląd. Dane ustawień i stabilne identyfikatory wariantów są przechowywane niezależnie od renderera, tak aby przyszły moduł wydruku/PDF mógł je zastosować. Podgląd jasno oznacza dane przykładowe. Ten zakres nie implementuje całego modułu faktur ani eksportu PDF.

### Dane i kopie zapasowe

- prezentacja rzeczywistej lokalizacji bazy zarządzanej przez aplikację;
- możliwość otwarcia folderu danych w systemie;
- ręczne utworzenie pełnej kopii i eksport pakietu kopii do wskazanego przez użytkownika pliku;
- przywracanie pełnej kopii z pliku, z walidacją i jawnym potwierdzeniem;
- automatyczna kopia bezpieczeństwa aktualnego stanu przed przywróceniem.

Lokalizacja bazy pozostaje zarządzana przez aplikację i nie jest dowolnie zmieniana w tej iteracji. Eksport i import kopii używa natywnego okna wyboru pliku. Nie ma synchronizacji chmurowej ani automatycznej wysyłki.

### Ustawienia aplikacji

- motyw: systemowy, jasny lub ciemny;
- podstawowe preferencje interfejsu, o ile mają bezpośrednie działanie w obecnym UI;
- wyświetlenie wersji aplikacji pobranej z metadanych builda, bez ręcznie utrzymywanej kopii numeru wersji.

## 3. Architektura i przepływ danych

1. Przy starcie backend otwiera lub inicjalizuje bazę w katalogu danych użytkownika.
2. Wersjonowane migracje tworzą schemat ustawień i tabelę wersji schematu. Inicjalizacja jest idempotentna.
3. Rust definiuje typy domenowe i walidację; komendy Tauri udostępniają odczyt/zapis ustawień, obsługę logo, informacje o lokalizacji, kopie i przywracanie.
4. Frontend pobiera pełny model ustawień, renderuje sekcje i zapisuje każdą sekcję przyciskiem zapisu z informacją o wyniku. Błędy walidacji backendu mapują się na odpowiednie pola.
5. Sekcje mogą zapisywać niezależnie, ale każda operacja aktualizacji pojedynczego zestawu ustawień jest atomowa. Błąd nie może pozostawić częściowo zapisanej sekcji.
6. Pozostałe moduły pobierają konfigurację przez warstwę domenową/komendy, a nie przez bezpośrednie odczyty z UI.

Dane rejestrowe i podatkowe powinny być modelowane jako opcjonalne wartości o jawnych typach/enumach, a nie zakodowane jako wymagane pola ani niejawne domyślne stawki. Dodanie pól i zmian schematu będzie realizowane kolejnymi migracjami.

## 4. Numeracja i niezmienność dokumentów

- Numeracja modelowana jest według stabilnego klucza typu dokumentu; format, prefix, tokeny okresu i licznik należą do konfiguracji serii.
- Numer jest przydzielany tylko w backendzie. Frontend nie wylicza ani nie rezerwuje numerów.
- Transakcja wystawienia obejmuje pobranie/licznik, zapis dokumentu i jego migawki oraz aktualizację licznika. Konflikt lub błąd wycofuje całą transakcję.
- Unikalny indeks/constraint na numerze w zakresie właściwym dla dokumentu zabezpiecza przed duplikatem także przy współbieżnych żądaniach.
- Ustawienia są wzorcem dla nowych dokumentów. Wystawiony dokument zachowuje własną migawkę danych sprzedawcy, nabywcy i danych dokumentu; edycja ustawień nie aktualizuje tej migawki.
- Szczegółowe typy dokumentów i reguły prawne nie są rozszerzane poza zatwierdzony zakres MVP.

## 5. Format i bezpieczne odtwarzanie kopii

Kopia będzie wersjonowanym archiwum ZIP aplikacji zawierającym bazę SQLite, manifest z identyfikatorem formatu, wersją schematu, wersją aplikacji i kontrolnymi sumami plików oraz wszystkie zarządzane zasoby (w tym logo). Ścieżki wpisów są logicznymi nazwami wewnątrz archiwum, a nie dowolnymi ścieżkami systemowymi. Format ma być przenośny i możliwy do walidacji przed użyciem; odczyt jest ograniczony limitami skompresowanego i rozpakowanego rozmiaru, a wpisy zduplikowane, nieoczekiwane lub wychodzące poza dozwolony układ są odrzucane.

Przywracanie przebiega w następującej kolejności:

1. Odczyt kopii bez modyfikowania bieżącej bazy.
2. Walidacja typu/wersji pakietu, kompletności, nazw i ścieżek wpisów, limitów rozmiaru, sum kontrolnych, integralności SQLite oraz obsługiwanej wersji schematu. Archiwum nie może umożliwić zapisania plików poza katalogiem tymczasowym aplikacji.
3. Wyświetlenie podsumowania kopii i żądanie wyraźnego potwierdzenia zastąpienia wszystkich danych.
4. Utworzenie i weryfikacja automatycznej kopii aktualnej bazy.
5. Przygotowanie odtworzonej bazy w pliku tymczasowym, sprawdzenie migracji/integralności i atomowa podmiana aktywnej bazy.
6. Jeśli podmiana nie powiedzie się, zachowanie bieżącej bazy; automatyczna kopia pozostaje dostępna do odzyskania.

Błędny format, niezgodna przyszła wersja schematu, obcięty plik, niepoprawny checksum lub nieprawidłowa integralność skutkuje przerwaniem przed punktem 3 i bez zapisu do aktywnej bazy. Komunikat ma wskazywać problem bez ujawniania wrażliwych danych.

## 6. Interfejs i walidacja

- Zachować polski język UI i istniejący kierunek wizualny: ciemne tło, grafitowe powierzchnie, pomarańcz `#FF6F00`, fiolet `#9C27B0`.
- Nawigacja sekcji ma być czytelna także przy mniejszej szerokości okna; formularze grupują powiązane dane.
- Każde pole ma etykietę, właściwy typ wejścia, komunikat błędu i widoczny stan zapisu.
- Walidacja syntaktyczna pól (np. e-mail, numer rachunku, format numeracji) nie ma udawać walidacji prawnej ani wymuszać pól opcjonalnych.
- Zmiany mają jasny stan: niezapisane, zapisane lub błąd; błędy zapisu nie mogą być prezentowane jako sukces.
- Usuwanie logo i przywracanie bazy wymagają jawnego działania; przywracanie dodatkowo wymaga potwierdzenia skutków.
- Podgląd wyglądu wykorzystuje aktualne ustawienia; przykładowe wartości w podglądzie muszą być wyraźnie oznaczone jako przykładowe, nie jako dane firmy.

## 7. Migracje, testy i kryteria akceptacji

### Migracje

- Każda zmiana struktury bazy otrzymuje numerowaną, testowalną migrację.
- Uruchomienie aplikacji na nowej bazie tworzy wymagany schemat i puste opcjonalne dane podatkowe/rejestrowe.
- Migracja istniejącej bazy nie może kasować ani nadpisywać danych.

### Testy

- Testy jednostkowe walidacji ustawień i formatowania numerów, w tym błędnych wzorów i granic licznika.
- Testy backendowe migracji, trwałości po ponownym otwarciu, transakcyjności zapisu i ochrony unikalności numerów.
- Testy kopii: prawidłowy round-trip, uszkodzenie, ucięcie, niezgodna wersja, checksum, ścieżki traversal i awaria przywracania bez utraty aktywnych danych.
- Testy regresyjne potwierdzające, że zmiana ustawień nie zmienia wcześniej zapisanej migawki dokumentu.
- UI smoke test: edycja, zapis, ponowne otwarcie aplikacji, walidacja oraz potwierdzenie restore.
- Dodać i skonfigurować test runner tylko w zakresie niezbędnym do testów tego modułu; brak obecnego skryptu `npm test` należy obsłużyć w planie implementacji.

### Kryteria akceptacji

1. Ustawienia można edytować, zapisać i odczytać po ponownym uruchomieniu.
2. Dane firmy i ustawienia domyślne są udostępniane przez typowany backend.
3. Dane NIP/REGON/VAT są opcjonalne, nie mają narzuconych wartości i nie blokują pracy z ustawieniami.
4. Numeracja jest konfigurowalna per obsługiwany typ, a backendowa transakcja i constraint uniemożliwiają podwójne przydzielenie.
5. Ustawienia wyglądu, widoczności i szablonu są zapisywane, a podgląd odzwierciedla ich wartości.
6. Można utworzyć i wyeksportować pełną kopię oraz wyświetlić lokalizację danych.
7. Import kopii najpierw waliduje plik, wyraźnie informuje o zastąpieniu całej bazy i przed zamianą tworzy kopię aktualnego stanu.
8. Nieprawidłowa kopia nie zmienia żadnego aktywnego rekordu.
9. Żadne ustawienie ani kopia nie jest wysyłana do zewnętrznej usługi.
10. Zmiana konfiguracji nie modyfikuje migawki istniejącego dokumentu.

## 8. Poza zakresem

- Pełny moduł faktur, wystawianie dokumentów, renderowanie i eksport PDF.
- Implementacja katalogu, klientów i historii dokumentów, poza uwzględnieniem ich przyszłej obecności w pełnej kopii bazy.
- KSeF, synchronizacja, konta użytkowników, chmura, płatności online i księgowość.
- Domyślne stawki, status VAT, dane rejestrowe lub porady o zgodności prawnej.
- Dowolna ścieżka bazy konfigurowana przez użytkownika.
- Usuwanie danych lub reset bazy bez osobnego, jawnego potwierdzenia i projektu.

## 9. Pliki i obszary przewidywane do implementacji

Dokładny podział zostanie określony w planie implementacji po akceptacji tej specyfikacji. Przewidywane obszary zmian:

- `src/features/settings/` — widok, sekcje formularza, typy i walidacja UI;
- `src-tauri/src/` — modele, inicjalizacja/migracje SQLite, repozytoria, serwisy i komendy Tauri;
- `src-tauri/migrations/` — wersjonowane migracje;
- `src-tauri/Cargo.toml` oraz `package.json`/lockfile — uzasadnione zależności i test runner;
- `src/App.tsx` / style aplikacji — podłączenie istniejącej sekcji Ustawienia;
- testy frontendowe i backendowe oraz ewentualnie README opisujące lokalizację danych i kopie.

Zależności będą dobierane dopiero w planie, po sprawdzeniu zgodności ze stabilnym Tauri 2 i istniejącym środowiskiem. Każda nowa zależność musi mieć uzasadnienie.
