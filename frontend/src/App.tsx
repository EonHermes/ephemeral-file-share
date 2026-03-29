import { useState } from 'react'
import axios from 'axios'
import { QRCodeSVG } from 'qrcode.react'

const API_URL = 'http://localhost:3000/api'

interface TransferResponse {
  id: string
  token: string
  qr_url: string
  expires_at?: string
}

function App() {
  const [filename, setFilename] = useState('')
  const [fileData, setFileData] = useState<File | null>(null)
  const [expiresIn, setExpiresIn] = useState(60)
  const [transfer, setTransfer] = useState<TransferResponse | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      setFileData(e.target.files[0])
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setLoading(true)
    setError('')

    try {
      if (!fileData) {
        throw new Error('Please select a file')
      }

      // Read file as base64
      const reader = new FileReader()
      const dataPromise = new Promise<string>((resolve, reject) => {
        reader.onload = () => resolve(reader.result as string)
        reader.onerror = () => reject(new Error('Failed to read file'))
        reader.readAsDataURL(fileData)
      })

      const base64Data = await dataPromise
      // Remove the data URL prefix (e.g., "data:image/png;base64,")
      const pureBase64 = base64Data.split(',')[1]

      const response = await axios.post<TransferResponse>(`${API_URL}/transfer`, {
        filename: fileData.name,
        data: pureBase64,
        expires_in_minutes: expiresIn,
      })

      setTransfer(response.data)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create transfer')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="container">
      <header>
        <h1>🔐 Ephemeral File Share</h1>
        <p>Secure, encrypted file transfers that self-destruct</p>
      </header>

      {!transfer ? (
        <form onSubmit={handleSubmit} className="upload-form">
          <div className="form-group">
            <label htmlFor="filename">File Name</label>
            <input
              type="text"
              id="filename"
              value={filename}
              onChange={(e) => setFilename(e.target.value)}
              placeholder="secret-document.txt"
              required
            />
          </div>

          <div className="form-group">
            <label htmlFor="file">Select File</label>
            <input
              type="file"
              id="file"
              onChange={handleFileChange}
              required
            />
          </div>

          <div className="form-group">
            <label htmlFor="expires">Expires In (minutes)</label>
            <select
              id="expires"
              value={expiresIn}
              onChange={(e) => setExpiresIn(Number(e.target.value))}
            >
              <option value={15}>15 minutes</option>
              <option value={30}>30 minutes</option>
              <option value={60}>1 hour</option>
              <option value={120}>2 hours</option>
              <option value={360}>6 hours</option>
            </select>
          </div>

          {error && <div className="error">{error}</div>}

          <button type="submit" disabled={loading} className="submit-btn">
            {loading ? 'Encrypting...' : 'Create Secure Transfer'}
          </button>
        </form>
      ) : (
        <div className="success">
          <h2>✅ Transfer Created!</h2>
          
          <div className="qr-section">
            <QRCodeSVG 
              value={`ephemeral://transfer/${transfer.token}`}
              size={256}
              level="H"
            />
          </div>

          <div className="info-card">
            <h3>Transfer Details</h3>
            <p><strong>ID:</strong> {transfer.id}</p>
            <p><strong>Token:</strong> <code>{transfer.token.substring(0, 20)}...</code></p>
            {transfer.expires_at && (
              <p><strong>Expires:</strong> {new Date(transfer.expires_at).toLocaleString()}</p>
            )}
          </div>

          <div className="share-section">
            <h3>Share This Link</h3>
            <code className="share-link">
              http://localhost:3000/api/qr/{transfer.token}
            </code>
            <button 
              onClick={() => navigator.clipboard.writeText(`http://localhost:3000/api/qr/${transfer.token}`)}
              className="copy-btn"
            >
              Copy Link
            </button>
          </div>

          <button onClick={() => setTransfer(null)} className="new-transfer-btn">
            Create Another Transfer
          </button>
        </div>
      )}

      <footer>
        <p>Built with Rust + React | End-to-End Encrypted</p>
      </footer>
    </div>
  )
}

export default App
