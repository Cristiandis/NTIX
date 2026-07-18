import({ "../../packages/winget.lua", "../../packages/chocolatey.lua" })

return {
    options = {
        winget = { enable = true, acceptAgreements = true },
        chocolatey = { enable = true, yes = true },
        scoop = { enable = true, buckets = { "main", "extras", "versions" } }
    }
}
