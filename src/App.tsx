import { useState } from "react";
import "./App.css";

type SectionId = "dashboard" | "invoices" | "customers" | "catalog" | "settings";

type Section = {
  id: SectionId;
  label: string;
  description: string;
};

const sections: Section[] = [
  { id: "dashboard", label: "Pulpit", description: "Przegląd aplikacji" },
  { id: "invoices", label: "Faktury", description: "Dokumenty sprzedaży" },
  { id: "customers", label: "Kontrahenci", description: "Dane nabywców" },
  { id: "catalog", label: "Katalog", description: "Usługi i produkty" },
  { id: "settings", label: "Ustawienia", description: "Dane firmy i preferencje" },
];

function NavIcon({ section }: { section: SectionId }) {
  const paths: Record<SectionId, string> = {
    dashboard: "M3 3h7v7H3z M14 3h7v4h-7z M14 10h7v11h-7z M3 14h7v7H3z",
    invoices: "M6 2h9l4 4v16H6z M14 2v5h5 M9 12h7 M9 16h7",
    customers: "M16 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2 M10 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8 M20 21v-2a4 4 0 0 0-3-3.87 M16 3.13a4 4 0 0 1 0 7.75",
    catalog: "M4 5.5A2.5 2.5 0 0 1 6.5 3H20v15H6.5A2.5 2.5 0 0 0 4 20.5z M4 5.5v15 M8 7h8 M8 11h8",
    settings: "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8 M19.4 15a1.7 1.7 0 0 0 .34 1.88l.06.06-1.7 2.94-.08-.02a1.7 1.7 0 0 0-1.76.5l-.06.06h-3.4l-.04-.08a1.7 1.7 0 0 0-1.43-1.02l-.08-.01-1.7-2.94.06-.06A1.7 1.7 0 0 0 10 14.5v-.08l-1.7-2.94.04-.07a1.7 1.7 0 0 0 .12-1.76l-.04-.07 1.7-2.94.08.02a1.7 1.7 0 0 0 1.76-.5l.06-.06h3.4l.04.08a1.7 1.7 0 0 0 1.43 1.02l.08.01 1.7 2.94-.06.06A1.7 1.7 0 0 0 18.5 12v.08l.9 1.56z",
  };

  return (
    <svg aria-hidden="true" viewBox="0 0 24 24" className="nav-icon" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round">
      <path d={paths[section]} />
    </svg>
  );
}

function App() {
  const [activeSection, setActiveSection] = useState<SectionId>("dashboard");
  const currentSection = sections.find((section) => section.id === activeSection) ?? sections[0];

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand-lockup" aria-label="Dual Zone Garage">
          <span className="brand-mark" aria-hidden="true">DZ</span>
          <span className="brand-name">Dual Zone <strong>Garage</strong></span>
        </div>

        <div className="sidebar-caption">Obszar roboczy</div>
        <nav className="primary-nav" aria-label="Nawigacja główna">
          {sections.map((section) => (
            <button
              className={`nav-item${section.id === activeSection ? " nav-item--active" : ""}`}
              key={section.id}
              type="button"
              aria-current={section.id === activeSection ? "page" : undefined}
              onClick={() => setActiveSection(section.id)}
            >
              <NavIcon section={section.id} />
              <span>{section.label}</span>
            </button>
          ))}
        </nav>

        <div className="sidebar-note">
          <span className="sidebar-note__mark" aria-hidden="true" />
          <p>Aplikacja rozwijana etapami</p>
        </div>
      </aside>

      <div className="workspace">
        <header className="topbar">
          <div className="topbar__context">
            <span className="topbar__product">Dual Zone Garage</span>
            <span className="topbar__divider" aria-hidden="true">/</span>
            <span>{currentSection.label}</span>
          </div>
          <div className="release-status">
            <span className="release-status__dot" aria-hidden="true" />
            Wersja startowa
          </div>
        </header>

        <main className="page-content" key={activeSection}>
          <div className="page-heading">
            <h1>{currentSection.label}</h1>
            <p>{currentSection.description}</p>
          </div>

          {activeSection === "dashboard" ? (
            <section className="welcome-panel" aria-labelledby="welcome-title">
              <div className="welcome-panel__accent" aria-hidden="true" />
              <div className="welcome-panel__content">
                <p className="welcome-panel__eyebrow">Fundament aplikacji</p>
                <h2 id="welcome-title">Witaj w Dual Zone Garage</h2>
                <p className="welcome-panel__copy">
                  To lokalny szkielet aplikacji do przygotowywania faktur. Kolejne moduły będą dodawane etapami.
                </p>
              </div>
              <div className="welcome-panel__stamp" aria-hidden="true">DZG</div>
            </section>
          ) : (
            <section className="planned-panel" aria-live="polite">
              <span className="planned-panel__line" aria-hidden="true" />
              <div>
                <h2>Moduł w przygotowaniu</h2>
                <p>Widok „{currentSection.label}” zostanie dodany w kolejnym etapie projektu.</p>
              </div>
            </section>
          )}
        </main>

        <footer className="workspace-footer">
          <span>Dual Zone Garage</span>
          <span>Fundament · etap 1</span>
        </footer>
      </div>
    </div>
  );
}

export default App;
