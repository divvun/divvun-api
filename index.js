require("dotenv").config() 
const express = require("express")
const { decorateApp } = require("@awaitjs/express")
const { spawn } = require("child_process")

const app = decorateApp(express())
app.use(express.json())

function runDivvunChecker(tag, text) {
  const p = spawn("docker", ["run", "-i", "divvun-gramcheck", `${tag}.zcheck`])
  
  return new Promise((resolve, reject) => {
    let out = ""
    let err = ""

    p.stdout.on('data', (data) => {
      out += data
    })
    
    p.stderr.on('data', (data) => {
      err += data
    })
    
    p.on('exit', (code) => {
      if (code != 0) {
        reject(new Error(err))
      } else {
        resolve(out)
      }
    })

    p.stdin.write(text)
    p.stdin.end()
  })
}

app.postAsync("/grammar/:tag", async (req, res) => {
  const { tag } = req.params
  const { text } = req.body

  res.setHeader('Content-Type', 'application/json')

  try {
    const out = await runDivvunChecker(tag, text)
    res.send(out)
  } catch (err) {
    if (err.message.startsWith("Archive path not OK")) {
      res.status(400)
      res.send(JSON.stringify({ error: "Language not supported" }))
    } else {
      res.status(500)
      res.send(JSON.stringify({ error: "Internal server error" }))
    }
  }
})

const port = process.env.PORT || 8000
const host = process.env.HOST || "127.0.0.1"
app.listen(port, host, () => console.log(`Listening on http://127.0.0.1:${port}`))
