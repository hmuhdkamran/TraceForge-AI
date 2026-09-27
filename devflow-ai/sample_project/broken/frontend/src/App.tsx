import { UploadForm } from './components/UploadForm';

export function App() {
  return (
    <div style={{ maxWidth: '800px', margin: '0 auto', padding: '2rem' }}>
      <header style={{ marginBottom: '2rem', borderBottom: '1px solid #e2e8f0', paddingBottom: '1rem' }}>
        <h1 style={{ fontSize: '1.875rem', fontWeight: 'bold', color: '#0f172a' }}>UploadLab</h1>
        <p style={{ color: '#64748b', marginTop: '0.25rem' }}>
          Multi-file upload application — ContractGuard Sample Testbed
        </p>
        <div style={{ marginTop: '0.5rem', display: 'inline-block', backgroundColor: '#fef2f2', border: '1px solid #fecaca', padding: '0.25rem 0.75rem', borderRadius: '0.375rem', color: '#991b1b', fontSize: '0.875rem' }}>
          ⚠️ BROKEN FIXTURE: Deliberate contract mismatch in request construction
        </div>
      </header>

      <main>
        <UploadForm />
      </main>
    </div>
  );
}

export default App;
