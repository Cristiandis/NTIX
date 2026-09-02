import({ "./base.lua" })

return {
    pkgs = {
        winget = {
            "Microsoft.VisualStudioCode",
            "Docker.DockerDesktop",
            "Postman.Postman",
            "JetBrains.IntelliJIDEA.Ultimate",
            "Microsoft.AzureCLI",
        },
        chocolatey = {
            "docker-desktop",
            "kubernetes-cli",
            "terraform",
            "helm",
        },
        scoop = {
            "nodejs-lts",
            "python",
            "go",
            "rust",
            "docker-cli",
            "kubectl",
            "helm",
        }
    }
}
