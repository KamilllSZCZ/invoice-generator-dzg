# Dual Zone Garage — generator faktur

## 1. Cel

Stworzyć od zera niewielką, lokalną aplikację do przygotowywania i obsługi faktur dla Dual Zone Garage. Projekt ma być eksperymentem i narzędziem do nauki, a nie próbą budowy pełnego programu księgowego. Priorytety: prostota, przewidywalność, poprawne obliczenia, czytelny interfejs i niski koszt dalszego rozwijania z Codexem.

Nie kopiujemy kodu ani architektury poprzedniego generatora Python. Nie migrujemy jego danych, chyba że zostanie to zlecone osobno.

## 2. Co oznacza „desktopowa aplikacja webowa”

Aplikacja będzie desktopowa: uruchamiana w osobnym oknie Windows, z instalatorem i skrótem. Jej interfejs powstanie w technologiach webowych (React, TypeScript, CSS), ale zostanie opakowany przez Tauri 2. Nie będzie publiczną stroną internetową i nie będzie wymagała hostingu ani własnego serwera.

## 3. Stos technologiczny

- **Tauri 2** — opakowanie desktopowe i dostęp do funkcji systemowych.
- **React + TypeScript + Vite** — interfejs aplikacji.
- **Rust** — niewielka warstwa backendowa Tauri, komendy aplikacji i dostęp do bazy.
- **SQLite** — lokalna baza danych.
- **CSS z tokenami projektu** — stylowanie interfejsu; bez dokładania rozbudowanego UI frameworka na starcie.
- **Vitest** — testy logiki frontendowej i obliczeń, jeśli będzie potrzebny.
- **PDF** — osobny moduł. Przed jego implementacją należy wybrać jedną bibliotekę/metodę, która działa w Tauri i poprawnie obsługuje polskie znaki oraz zapis pliku.

Używaj aktualnych, stabilnych wersji zgodnych ze sobą. Nie aktualizuj zależności bez potrzeby.

## 4. Zakres pierwszej wersji (MVP)

1. **Konfiguracja firmy** — nazwa, adres, NIP i pozostałe dane sprzedawcy; status VAT i ustawienia dokumentów konfigurowalne, bez zakładania ich z góry.
2. **Kontrahenci** — dodawanie, edycja, podgląd, wyszukiwanie i usuwanie z kontrolą powiązań z dokumentami.
3. **Katalog pozycji** — usługi i produkty z nazwą, jednostką, ceną domyślną i przypisaną stawką/oznaczeniem podatku.
4. **Faktury** — tworzenie wersji roboczej, dodawanie pozycji, ilości, cen, rabatu (jeżeli zostanie uwzględniony w projekcie formularza), termin i metoda płatności, daty dokumentu i sprzedaży.
5. **Obliczenia** — wartości pozycji oraz sumy netto, podatku i brutto, z jedną centralną implementacją i testami.
6. **Wystawienie dokumentu** — numer z konfigurowalnej serii, zapis dokumentu i jego pozycji w jednej transakcji.
7. **PDF** — podgląd i zapis faktury do pliku PDF.
8. **Historia faktur** — lista, wyszukiwanie, filtrowanie po numerze, kontrahencie i dacie oraz podgląd dokumentu.
9. **Kopie zapasowe** — ręczny eksport bazy/danych i odtworzenie kopii z walidacją. Użytkownik powinien wiedzieć, gdzie przechowywana jest baza.

MVP powinno być możliwie małe. Dashboard może pokazywać wyłącznie proste podsumowanie i ostatnie dokumenty. Rozbudowane raporty zostają na później.

## 5. Poza zakresem MVP

- integracja z KSeF i wysyłka dokumentów,
- synchronizacja chmurowa i wielu użytkowników,
- logowanie i role użytkowników,
- magazyn, stany części i rezerwacje,
- płatności online i integracje bankowe,
- pełna księgowość, JPK i deklaracje podatkowe,
- aplikacja mobilna,
- automatyczne pobieranie danych z zewnętrznych serwisów.

Architektura może pozostawić miejsce na przyszły moduł KSeF, ale nie wolno przedstawiać go jako gotowego ani implementować „na zapas”.

## 6. Zasady dotyczące dokumentów i danych

- Dane firmy, kontrahentów i dokumentów są przechowywane lokalnie.
- Baza znajduje się w katalogu danych aplikacji użytkownika, nie w katalogu instalacyjnym.
- Wystawiony dokument przechowuje własny obraz danych sprzedawcy, nabywcy, pozycji, cen i stawek podatku z chwili wystawienia. Zmiana kartoteki nie zmienia starej faktury.
- Wersja robocza jest edytowalna. Wystawiona faktura jest niezmienna w zwykłym przepływie aplikacji.
- Numeracja jest konfigurowalna. Przydział numeru i wystawienie muszą być wykonywane atomowo.
- Kwoty liczone są precyzyjnie. Nie wolno opierać obliczeń finansowych na zwykłej arytmetyce zmiennoprzecinkowej JavaScript.
- Stawki i status VAT nie mogą być zaszyte na stałe w logice. Aplikacja musi umożliwiać ich konfigurację. Wartości początkowe należy ustalić z użytkownikiem.
- Przed użyciem produkcyjnym użytkownik powinien zweryfikować zakres danych i sposób wystawiania dokumentów z księgowym. Sam program nie gwarantuje zgodności podatkowej.

## 7. Proponowany podział kodu

```text
src/
  app/                  # start aplikacji, routing, layout
  features/
    dashboard/
    invoices/           # formularz, lista, podgląd
    customers/
    catalog/
    settings/
  shared/
    components/         # wspólne elementy interfejsu
    types/              # typy używane w frontendzie
    utils/
  styles/               # kolory, typografia, globalne style

src-tauri/
  src/
    commands/           # jawne komendy wywoływane przez frontend
    db/                 # połączenie, migracje i repozytoria SQLite
    models/             # modele backendu
    services/           # operacje biznesowe wymagające backendu
  migrations/           # wersjonowane migracje bazy

tests/                  # testy integracyjne, jeśli będą potrzebne
```

Nie twórz wszystkich katalogów na zapas, jeżeli nie mają jeszcze kodu. Rozwijaj strukturę wraz z funkcjami. Unikaj monolitycznych plików i nadmiernego rozdrabniania.

## 8. Proponowana nawigacja

- Pulpit
- Faktury
- Kontrahenci
- Katalog usług i produktów
- Ustawienia

Każda pozycja nawigacji ma prowadzić do działającego widoku albo być wyraźnie oznaczona jako planowana. Bez pustych przycisków udających funkcje.

## 9. Etapy realizacji

1. **Fundament** — sprawdzenie środowiska, utworzenie Tauri + React + TypeScript + Vite, uruchomienie pustej aplikacji i bazowego layoutu.
2. **Model danych** — projekt SQLite, migracja początkowa, inicjalizacja bazy w katalogu danych aplikacji i test połączenia.
3. **Ustawienia i kartoteki** — dane firmy, kontrahenci oraz katalog pozycji.
4. **Silnik faktur** — modele, walidacja, obliczenia i testy zaokrągleń.
5. **Wystawianie faktur** — formularz, wersje robocze, numeracja, transakcje i niezmienność wystawionych dokumentów.
6. **PDF i historia** — szablon PDF, zapis pliku, lista, wyszukiwanie i podgląd.
7. **Kopie zapasowe i testy całości** — eksport/odtworzenie, testy scenariuszy i sprawdzenie instalatora.
8. **Wydanie testowe** — instalator Windows, instrukcja uruchomienia i lista znanych ograniczeń.

Każdy etap kończy się działającą wersją, testami i krótką aktualizacją dokumentacji. Nie przechodź do kolejnego etapu, jeśli podstawowy scenariusz bieżącego etapu nie działa.

## 10. Kryteria ukończenia MVP

- Aplikacja uruchamia się jako program Windows i działa bez internetu.
- Dane pozostają po zamknięciu i ponownym uruchomieniu aplikacji.
- Można skonfigurować dane firmy, dodać kontrahenta i pozycję katalogową.
- Można przygotować wersję roboczą i wystawić fakturę z unikalnym numerem.
- Obliczenia są wykonywane w jednym module i objęte testami.
- Zmiana danych kontrahenta nie zmienia wystawionego wcześniej dokumentu.
- Można wygenerować i zapisać czytelny PDF z polskimi znakami.
- Można znaleźć wystawiony dokument w historii.
- Można wykonać kopię zapasową i odtworzyć ją w kontrolowany sposób.
- Build produkcyjny i instalator przechodzą test ręczny na Windows.

## 11. Dokumentacja referencyjna

- Tauri — tworzenie projektu: https://v2.tauri.app/start/create-project/
- Tauri — czym jest framework: https://v2.tauri.app/start/
- Tauri — konfiguracja frontendu i rekomendacja Vite: https://v2.tauri.app/start/frontend/
