-- config.lua (root entry point)

-- Select environment
local env = os.getenv("NTIX_ENV") or "work.dev"

if env == "work.dev" then
    import("environments/work/dev.lua")
elseif env == "work.gaming" then
    import("environments/work/gaming.lua")
elseif env == "personal.gaming" then
    import("environments/personal/gaming.lua")
elseif env == "ci.minimal" then
    import("environments/ci/minimal.lua")
else
    error("Unknown environment: " .. env)
end

return { options = options, pkgs = pkgs }
