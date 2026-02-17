pub struct TlsProfile {
    pub user_agent: &'static str,
    pub browser: &'static str,
    pub platform: &'static str,
    pub ja3_hash: &'static str,
    pub http2_fingerprint: &'static str,
}

pub const TLS_PROFILES: &[TlsProfile] = &[
    // Chrome 131 Windows
    TlsProfile {
        user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
        browser: "Chrome",
        platform: "Desktop",
        ja3_hash: "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24,0",
        http2_fingerprint: "1:65536;2:0;3:1000;4:6291456;6:262144|15663105|0|m,a,s,p",
    },
    // Chrome 131 Mac
    TlsProfile {
        user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
        browser: "Chrome",
        platform: "Desktop",
        ja3_hash: "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24,0",
        http2_fingerprint: "1:65536;2:0;3:1000;4:6291456;6:262144|15663105|0|m,a,s,p",
    },
    // Chrome 130 Windows
    TlsProfile {
        user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
        browser: "Chrome",
        platform: "Desktop",
        ja3_hash: "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24,0",
        http2_fingerprint: "1:65536;2:0;3:1000;4:6291456;6:262144|15663105|0|m,a,s,p",
    },
    // Chrome Linux
    TlsProfile {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
        browser: "Chrome",
        platform: "Desktop",
        ja3_hash: "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24,0",
        http2_fingerprint: "1:65536;2:0;3:1000;4:6291456;6:262144|15663105|0|m,a,s,p",
    },
    // Firefox 133 Windows
    TlsProfile {
        user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0",
        browser: "Firefox",
        platform: "Desktop",
        ja3_hash: "771,4865-4867-4866-49195-49199-52393-52392-49196-49200-49162-49161-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-34-51-43-13-45-28-21,29-23-24-25-256-257,0",
        http2_fingerprint: "1:65536;4:131072;5:16384|12517377|3:0:0:201,5:0:0:1,7:0:0:1,9:0:7:1,11:0:3:1,13:0:0:241|m,p,a,s",
    },
    // Firefox 133 Mac
    TlsProfile {
        user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:133.0) Gecko/20100101 Firefox/133.0",
        browser: "Firefox",
        platform: "Desktop",
        ja3_hash: "771,4865-4867-4866-49195-49199-52393-52392-49196-49200-49162-49161-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-34-51-43-13-45-28-21,29-23-24-25-256-257,0",
        http2_fingerprint: "1:65536;4:131072;5:16384|12517377|3:0:0:201,5:0:0:1,7:0:0:1,9:0:7:1,11:0:3:1,13:0:0:241|m,p,a,s",
    },
    // Firefox 132 Windows
    TlsProfile {
        user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:132.0) Gecko/20100101 Firefox/132.0",
        browser: "Firefox",
        platform: "Desktop",
        ja3_hash: "771,4865-4867-4866-49195-49199-52393-52392-49196-49200-49162-49161-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-34-51-43-13-45-28-21,29-23-24-25-256-257,0",
        http2_fingerprint: "1:65536;4:131072;5:16384|12517377|3:0:0:201,5:0:0:1,7:0:0:1,9:0:7:1,11:0:3:1,13:0:0:241|m,p,a,s",
    },
    // Firefox Linux
    TlsProfile {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64; rv:133.0) Gecko/20100101 Firefox/133.0",
        browser: "Firefox",
        platform: "Desktop",
        ja3_hash: "771,4865-4867-4866-49195-49199-52393-52392-49196-49200-49162-49161-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-34-51-43-13-45-28-21,29-23-24-25-256-257,0",
        http2_fingerprint: "1:65536;4:131072;5:16384|12517377|3:0:0:201,5:0:0:1,7:0:0:1,9:0:7:1,11:0:3:1,13:0:0:241|m,p,a,s",
    },
    // Safari 18.1 Mac
    TlsProfile {
        user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Safari/605.1.15",
        browser: "Safari",
        platform: "Desktop",
        ja3_hash: "771,4865-4866-4867-49196-49195-52393-49200-49199-52392-49162-49161-49172-49171-157-156-53-47-49160-49170-10,0-23-65281-10-11-16-5-13-18-51-45-43-27-21,29-23-24-25,0",
        http2_fingerprint: "4:4194304;3:100|10485760|0|m,s,p,a",
    },
    // Safari 17.6 Mac
    TlsProfile {
        user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.6 Safari/605.1.15",
        browser: "Safari",
        platform: "Desktop",
        ja3_hash: "771,4865-4866-4867-49196-49195-52393-49200-49199-52392-49162-49161-49172-49171-157-156-53-47-49160-49170-10,0-23-65281-10-11-16-5-13-18-51-45-43-27-21,29-23-24-25,0",
        http2_fingerprint: "4:4194304;3:100|10485760|0|m,s,p,a",
    },
    // Edge 131 Windows
    TlsProfile {
        user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0",
        browser: "Edge",
        platform: "Desktop",
        ja3_hash: "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24,0",
        http2_fingerprint: "1:65536;2:0;3:1000;4:6291456;6:262144|15663105|0|m,a,s,p",
    },
    // Edge 130 Windows
    TlsProfile {
        user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0",
        browser: "Edge",
        platform: "Desktop",
        ja3_hash: "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24,0",
        http2_fingerprint: "1:65536;2:0;3:1000;4:6291456;6:262144|15663105|0|m,a,s,p",
    },
    // Mobile Chrome Android
    TlsProfile {
        user_agent: "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36",
        browser: "Chrome",
        platform: "Mobile",
        ja3_hash: "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24,0",
        http2_fingerprint: "1:65536;2:0;3:1000;4:6291456;6:262144|15663105|0|m,a,s,p",
    },
    // Mobile Chrome Samsung
    TlsProfile {
        user_agent: "Mozilla/5.0 (Linux; Android 14; SM-S921B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36",
        browser: "Chrome",
        platform: "Mobile",
        ja3_hash: "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24,0",
        http2_fingerprint: "1:65536;2:0;3:1000;4:6291456;6:262144|15663105|0|m,a,s,p",
    },
    // Mobile Safari iOS 18
    TlsProfile {
        user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Mobile/15E148 Safari/604.1",
        browser: "Safari",
        platform: "Mobile",
        ja3_hash: "771,4865-4866-4867-49196-49195-52393-49200-49199-52392-49162-49161-49172-49171-157-156-53-47-49160-49170-10,0-23-65281-10-11-16-5-13-18-51-45-43-27-21,29-23-24-25,0",
        http2_fingerprint: "4:4194304;3:100|10485760|0|m,s,p,a",
    },
    // Mobile Safari iOS 17
    TlsProfile {
        user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 17_7 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.7 Mobile/15E148 Safari/604.1",
        browser: "Safari",
        platform: "Mobile",
        ja3_hash: "771,4865-4866-4867-49196-49195-52393-49200-49199-52392-49162-49161-49172-49171-157-156-53-47-49160-49170-10,0-23-65281-10-11-16-5-13-18-51-45-43-27-21,29-23-24-25,0",
        http2_fingerprint: "4:4194304;3:100|10485760|0|m,s,p,a",
    },
    // Mobile Chrome iOS
    TlsProfile {
        user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/131.0.6778.73 Mobile/15E148 Safari/604.1",
        browser: "Chrome",
        platform: "Mobile",
        ja3_hash: "771,4865-4866-4867-49196-49195-52393-49200-49199-52392-49162-49161-49172-49171-157-156-53-47-49160-49170-10,0-23-65281-10-11-16-5-13-18-51-45-43-27-21,29-23-24-25,0",
        http2_fingerprint: "4:4194304;3:100|10485760|0|m,s,p,a",
    },
    // Firefox Android
    TlsProfile {
        user_agent: "Mozilla/5.0 (Android 14; Mobile; rv:133.0) Gecko/133.0 Firefox/133.0",
        browser: "Firefox",
        platform: "Mobile",
        ja3_hash: "771,4865-4867-4866-49195-49199-52393-52392-49196-49200-49162-49161-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-34-51-43-13-45-28-21,29-23-24-25-256-257,0",
        http2_fingerprint: "1:65536;4:131072;5:16384|12517377|3:0:0:201,5:0:0:1,7:0:0:1,9:0:7:1,11:0:3:1,13:0:0:241|m,p,a,s",
    },
    // iPad Safari
    TlsProfile {
        user_agent: "Mozilla/5.0 (iPad; CPU OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Mobile/15E148 Safari/604.1",
        browser: "Safari",
        platform: "Mobile",
        ja3_hash: "771,4865-4866-4867-49196-49195-52393-49200-49199-52392-49162-49161-49172-49171-157-156-53-47-49160-49170-10,0-23-65281-10-11-16-5-13-18-51-45-43-27-21,29-23-24-25,0",
        http2_fingerprint: "4:4194304;3:100|10485760|0|m,s,p,a",
    },
    // Chrome Android Tablet
    TlsProfile {
        user_agent: "Mozilla/5.0 (Linux; Android 14; SM-X710) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
        browser: "Chrome",
        platform: "Desktop",
        ja3_hash: "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24,0",
        http2_fingerprint: "1:65536;2:0;3:1000;4:6291456;6:262144|15663105|0|m,a,s,p",
    },
];
