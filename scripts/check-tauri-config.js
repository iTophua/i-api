import { exec } from 'child_process'
import { promisify } from 'util'

const execAsync = promisify(exec)

async function checkNsisSchema() {
  console.log('Checking Tauri NSIS configuration schema...\n')
  
  try {
    const { stdout } = await execAsync('npm exec tauri build -- --help | head -20')
    console.log(stdout)
  } catch (error) {
    console.log('Error:', error.message)
  }
}

checkNsisSchema()
