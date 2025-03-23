import iinaIcon from '@/assets/iina.png'
import infuseIcon from '@/assets/infuse.png'
import vlcIcon from '@/assets/vlc.png'
import mpvIcon from '@/assets/mpv.png'

export interface Player {
  name: string
  icon: string
  scheme: string
  generatePlayUrl(videoUrl: string): string
}

export const players: Player[] = [
  {
    name: 'IINA',
    icon: iinaIcon,
    scheme: 'iina://weblink?url=',
    generatePlayUrl(videoUrl: string) {
      return `${this.scheme}${encodeURIComponent(videoUrl)}`
    }
  },
  {
    name: 'Infuse',
    icon: infuseIcon,
    scheme: 'infuse://x-callback-url/play?url=',
    generatePlayUrl(videoUrl: string) {
      return `${this.scheme}${encodeURIComponent(videoUrl)}`
    }
  },
  {
    name: 'VLC',
    icon: vlcIcon,
    scheme: 'vlc-x-callback://x-callback-url/stream?url=',
    generatePlayUrl(videoUrl: string) {
      return `${this.scheme}${encodeURIComponent(videoUrl)}`
    }
  },
  {
    name: 'MPV',
    icon: mpvIcon,
    scheme: 'mpv://',
    generatePlayUrl(videoUrl: string) {
      return `${this.scheme}${encodeURIComponent(videoUrl)}`
    }
  }
]