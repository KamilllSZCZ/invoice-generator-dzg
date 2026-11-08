## Zasady pracy

1. Przed zadaniem przeczytaj ten plik oraz odpowiedni fragment `PROJECT_PLAN.md`. Nie analizuj ponownie całego repozytorium bez konkretnej potrzeby.
2. Realizuj wyłącznie zakres bieżącego zadania. Nie dodawaj „przy okazji” funkcji ani refaktoryzacji.
3. Najpierw wskaż krótko pliki, które zamierzasz zmienić. Następnie wykonaj zadanie.
4. Nie przepisuj działających modułów bez wyraźnej potrzeby. Nie zmieniaj architektury samodzielnie.
5. Utrzymuj podział na funkcje i warstwy. Nie twórz jednego dużego pliku. Nie rozbijaj też kodu na dziesiątki drobnych plików bez uzasadnienia.
6. Nie dodawaj biblioteki, jeżeli zadanie można rozsądnie wykonać bez niej. Każdą nową zależność uzasadnij.
7. Nie twórz fikcyjnych funkcji, przycisków ani danych produkcyjnych. Element, który jeszcze nie działa, oznacz jako niezaimplementowany lub pozostaw poza interfejsem.
8. Nie wysyłaj danych firmy, klientów ani faktur do usług zewnętrznych. Aplikacja ma działać lokalnie i offline, dopóki osobne zadanie nie określi inaczej.
9. Nie zapisuj sekretów, haseł, kluczy ani certyfikatów w repozytorium.

## Architektura i jakość

- Frontend: React + TypeScript + Vite.
- Desktop: Tauri 2.
- Trwałe dane: SQLite, obsługiwany przez warstwę Rust/Tauri. Frontend nie wykonuje dowolnych zapytań SQL.
- Komunikacja frontend–backend odbywa się przez jawne, typowane komendy Tauri.
- Logikę obliczeń faktur trzymaj poza komponentami UI. Musi mieć testy jednostkowe.
- Nie używaj zwykłego `number` do obliczeń kwot pieniężnych. Użyj precyzyjnej reprezentacji dziesiętnej, ustal jedną regułę zaokrąglania i zapisuj kwoty w bazie jako całkowitą liczbę groszy.
- Dane kontrahenta i sprzedawcy na wystawionej fakturze muszą być migawką z chwili wystawienia. Późniejsza edycja kartoteki nie może zmieniać historycznego dokumentu.
- Wystawiony dokument nie może być po cichu edytowany ani nadpisywany. Zmiany dokumentów wystawionych wymagają osobnego, jawnego procesu zgodnego z przyjętymi zasadami.
- Każda zmiana struktury SQLite wymaga wersjonowanej migracji.
- Operacja wystawienia faktury i przydzielenia numeru musi być atomowa oraz chronić przed duplikatami numerów.
- Dane i ustawienia muszą przetrwać aktualizację aplikacji. Nie przechowuj bazy w katalogu instalacyjnym programu.

## Wygląd

- Kierunek wizualny Dual Zone Garage: ciemne tło, grafitowe powierzchnie, pomarańczowy akcent `#FF6F00`, fioletowy akcent `#9C27B0`.
- Interfejs ma być czytelny i użytkowy. Bez nadmiaru gradientów, animacji i ozdobników.
- Zachowaj spójne odstępy, typografię, stany formularzy, komunikaty błędów i potwierdzenia działań.
- Interfejs po polsku. Kod, nazwy zmiennych i komentarze techniczne po angielsku.

## Weryfikacja po zmianach

1. Uruchom testy dotyczące zmienionego modułu.
2. Uruchom TypeScript/build oraz odpowiednią kontrolę Rust, jeżeli zmieniano backend.
3. Nie deklaruj, że test przeszedł, jeśli go nie uruchomiłeś.
4. Nie ukrywaj błędów ani ostrzeżeń. Wymień znane problemy.
5. Na końcu podaj: co zostało zrobione, jakie pliki zmieniono, jakie testy uruchomiono i co pozostaje do zrobienia.

## Ograniczenie zakresu

Nie implementuj KSeF, synchronizacji chmurowej, logowania użytkowników, magazynu, płatności online ani rozbudowanej księgowości bez osobnego zadania. Nie zakładaj statusu VAT firmy, stawek podatku ani danych rejestrowych. Te wartości muszą być konfigurowalne i potwierdzone przez użytkownika.
