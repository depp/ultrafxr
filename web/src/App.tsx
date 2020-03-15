import React from 'react';
import { playSound } from './audio';

function App() {
  return (
    <div className="App">
      <button onClick={playSound}>Play</button>
    </div>
  );
}

export default App;
