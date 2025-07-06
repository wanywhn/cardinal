import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { once, listen } from '@tauri-apps/api/event';
import { List, AutoSizer } from 'react-virtualized';
import 'react-virtualized/styles.css';
import "./App.css";

function App() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState([]);
  const [isInitialized, setIsInitialized] = useState(false);
  const [isStatusBarVisible, setIsStatusBarVisible] = useState(true);
  const [statusText, setStatusText] = useState("Walking filesystem...");

  useEffect(() => {
    listen('status_update', (event) => {
      setStatusText(event.payload);
    });
    once('init_completed', () => {
      setIsInitialized(true);
    });
  }, []);

  useEffect(() => {
    if (isInitialized) {
      const timer = setTimeout(() => {
        setIsStatusBarVisible(false);
      }, 2000);
      return () => clearTimeout(timer);
    }
  }, [isInitialized]);

  useEffect(() => {
    const handleSearch = async () => {
      if (query.trim() === '') {
        setResults([]);
        return;
      }
      const searchResults = await invoke("search", { query });
      setResults(searchResults);
    };

    const timer = setTimeout(() => {
      handleSearch();
    }, 300); // 300ms debounce

    return () => clearTimeout(timer);
  }, [query]);

  const rowRenderer = ({ key, index, style }) => {
    return (
      <div key={key} style={style} className="row">
        {results[index]}
      </div>
    );
  };

  return (
    <main className="container">
      <div className="search-container">
        <input
          id="search-input"
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search for files and folders..."
          spellCheck={false}
          autoCorrect="off"
          autoComplete="off"
          autoCapitalize="off"
        />
      </div>
      <div className="results-container" style={{ flex: 1 }}>
        <AutoSizer>
          {({ height, width }) => (
            <List
              width={width}
              height={height}
              rowCount={results.length}
              rowHeight={30} // Adjust row height as needed
              rowRenderer={rowRenderer}
            />
          )}
        </AutoSizer>
      </div>
      {isStatusBarVisible && (
        <div className={`status-bar ${isInitialized ? 'fade-out' : ''}`}>
          {isInitialized ? 'Initialized' : 
            <div className="initializing-container">
              <div className="spinner"></div>
              <span>{statusText}</span>
            </div>
          }
        </div>
      )}
    </main>
  );
}

export default App;
