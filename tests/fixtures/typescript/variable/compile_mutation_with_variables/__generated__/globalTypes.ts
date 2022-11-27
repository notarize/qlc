/**
 * An OS makes hardware useful
 */
export enum OperatingSystem {
  ARCH_LINUX = "ARCH_LINUX",
  FREEBSD = "FREEBSD",
  UBUNTU_LINUX = "UBUNTU_LINUX",
}

export type ProvisionHostInput = {
  os: OperatingSystem;
};
